use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::AppConfig;

/// Statut de la clé SSH
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SshKeyStatus {
    /// Clé déjà dans l'agent, prête à utiliser
    InAgent,
    /// Clé existe mais nécessite une passphrase
    NeedsPassphrase { key_path: String },
    /// Pas de clé trouvée
    NoKey { expected_path: String },
    /// Agent SSH non disponible
    NoAgent,
}

/// Récupère le `SSH_AUTH_SOCK` depuis l'environnement (pour ssh-agent)
fn get_ssh_auth_sock() -> Option<String> {
    std::env::var("SSH_AUTH_SOCK").ok()
}

/// Expand ~ to home directory
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen('~', &home, 1);
        }
    }
    path.to_string()
}

/// Parse ~/.ssh/config to find `IdentityFile` for a given host
fn get_identity_file_from_ssh_config(host: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_path = format!("{home}/.ssh/config");
    let content = std::fs::read_to_string(&config_path).ok()?;

    let mut in_matching_host = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check for Host directive
        if trimmed.to_lowercase().starts_with("host ") {
            let hosts: Vec<&str> = trimmed[5..].split_whitespace().collect();
            in_matching_host = hosts.iter().any(|h| *h == host || *h == "*");
            continue;
        }

        // If we're in the matching host block, look for IdentityFile
        if in_matching_host && trimmed.to_lowercase().starts_with("identityfile ") {
            let identity_file = trimmed[13..].trim();
            return Some(expand_tilde(identity_file));
        }
    }

    None
}

/// Determine the SSH key path (from config or auto-detected from ~/.ssh/config)
fn get_ssh_key_path(config: &AppConfig) -> String {
    // If key_path is explicitly set and not the default, use it
    let default_key = format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_default());

    if config.ssh.key_path != default_key && !config.ssh.key_path.is_empty() {
        return expand_tilde(&config.ssh.key_path);
    }

    // Try to get from ~/.ssh/config
    if let Some(identity_file) = get_identity_file_from_ssh_config(&config.ssh.host) {
        tracing::info!("Auto-detected SSH key from config: {}", identity_file);
        return identity_file;
    }

    // Fall back to default
    expand_tilde(&config.ssh.key_path)
}

/// Vérifie si la clé SSH est dans l'agent
pub async fn check_key_in_agent(config: &AppConfig) -> SshKeyStatus {
    // Vérifier que l'agent SSH est disponible
    let Some(auth_sock) = get_ssh_auth_sock() else {
        return SshKeyStatus::NoAgent;
    };

    // Auto-detect key path from ~/.ssh/config or use config
    let key_path = get_ssh_key_path(config);
    tracing::info!("Checking SSH key: {}", key_path);

    // Vérifier que le fichier de clé existe
    if !std::path::Path::new(&key_path).exists() {
        return SshKeyStatus::NoKey {
            expected_path: key_path,
        };
    }

    // Lister les clés dans l'agent
    let output = Command::new("ssh-add")
        .arg("-l")
        .env("SSH_AUTH_SOCK", &auth_sock)
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Extraire le fingerprint de notre clé
            if let Ok(fp_output) = Command::new("ssh-keygen")
                .args(["-lf", &key_path])
                .output()
                .await
            {
                let fp_line = String::from_utf8_lossy(&fp_output.stdout);
                // Le fingerprint est le 2ème champ (ex: SHA256:xxx)
                if let Some(fingerprint) = fp_line.split_whitespace().nth(1) {
                    if stdout.contains(fingerprint) {
                        return SshKeyStatus::InAgent;
                    }
                }
            }
            SshKeyStatus::NeedsPassphrase { key_path }
        }
        Err(_) => SshKeyStatus::NoAgent,
    }
}

/// Ajoute une clé SSH à l'agent avec la passphrase fournie
/// Utilise `SSH_ASKPASS` avec un script temporaire (méthode OpenSSH 8.4+)
pub async fn add_key_to_agent(config: &AppConfig, passphrase: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let auth_sock =
        get_ssh_auth_sock().ok_or("SSH_AUTH_SOCK non défini. L'agent SSH n'est pas démarré.")?;

    // Auto-detect key path
    let key_path = get_ssh_key_path(config);
    tracing::info!("Adding SSH key to agent: {}", key_path);

    // Vérifier que le fichier existe
    if !std::path::Path::new(&key_path).exists() {
        return Err(format!("Clé SSH non trouvée: {key_path}"));
    }

    // Créer un script temporaire SSH_ASKPASS
    // Ce script sera exécuté par ssh-add pour obtenir la passphrase
    let temp_dir = std::env::temp_dir();
    let askpass_script = temp_dir.join(format!("ssh-askpass-{}", std::process::id()));

    // Échapper les guillemets simples dans la passphrase
    let escaped_passphrase = passphrase.replace('\'', "'\"'\"'");
    let script_content = format!("#!/bin/sh\necho '{escaped_passphrase}'\n");

    // Écrire le script
    std::fs::write(&askpass_script, &script_content)
        .map_err(|e| format!("Erreur création script SSH_ASKPASS: {e}"))?;

    // Rendre le script exécutable (700 = rwx------)
    std::fs::set_permissions(&askpass_script, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("Erreur permissions script: {e}"))?;

    let askpass_path = askpass_script.to_string_lossy().to_string();
    tracing::info!("Created SSH_ASKPASS script: {}", askpass_path);

    // Exécuter ssh-add avec SSH_ASKPASS
    // - DISPLAY="dummy" : nécessaire pour que SSH_ASKPASS soit utilisé
    // - SSH_ASKPASS_REQUIRE="force" : force l'utilisation de SSH_ASKPASS (OpenSSH 8.4+)
    // - stdin=null : important pour que ssh-add n'essaie pas de lire depuis le terminal
    let output = Command::new("ssh-add")
        .arg(&key_path)
        .env("SSH_AUTH_SOCK", &auth_sock)
        .env("DISPLAY", "dummy")
        .env("SSH_ASKPASS", &askpass_path)
        .env("SSH_ASKPASS_REQUIRE", "force")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Erreur lancement ssh-add: {e}"))?;

    // Supprimer le script temporaire (sécurité)
    let _ = std::fs::remove_file(&askpass_script);

    if output.status.success() {
        tracing::info!("Clé SSH ajoutée à l'agent: {}", key_path);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("ssh-add stderr: {}", stderr);

        // Message d'erreur plus clair pour passphrase incorrecte
        if stderr.contains("Bad passphrase") || stderr.contains("incorrect passphrase") {
            Err("Passphrase incorrecte".to_string())
        } else if stderr.contains("Could not open") {
            Err(format!("Impossible d'ouvrir la clé: {key_path}"))
        } else {
            Err(format!("Erreur ssh-add: {stderr}"))
        }
    }
}

/// Chemin du socket `ControlMaster`
fn control_socket_path(config: &AppConfig) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(format!(
        "{}/.ssh/benchmark-runner-{}.sock",
        home, config.ssh.host
    ))
}

/// Initialise une connexion `ControlMaster` persistante
pub async fn init_control_master(config: &AppConfig) -> Result<String, String> {
    let socket_path = control_socket_path(config);
    let socket_str = socket_path.to_string_lossy().to_string();

    // Supprimer l'ancien socket s'il existe
    let _ = std::fs::remove_file(&socket_path);

    let host = format!("{}@{}", config.ssh.user, config.ssh.host);

    let mut command = Command::new("ssh");
    command.args([
        "-o",
        "ControlMaster=yes",
        "-o",
        &format!("ControlPath={socket_str}"),
        "-o",
        "ControlPersist=yes",
        "-N",
        "-f",
        &host,
    ]);

    if let Some(auth_sock) = get_ssh_auth_sock() {
        command.env("SSH_AUTH_SOCK", auth_sock);
    }

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| format!("Erreur init ControlMaster: {e}"))?;

    if output.status.success() {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        if socket_path.exists() {
            tracing::info!("SSH ControlMaster prêt: {}", socket_str);
            Ok(socket_str)
        } else {
            Err("Socket ControlMaster non créé".to_string())
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("ControlMaster échoué: {stderr}"))
    }
}

/// Ferme la connexion `ControlMaster`
pub async fn close_control_master(config: &AppConfig) -> Result<(), String> {
    let socket_path = control_socket_path(config);
    let socket_str = socket_path.to_string_lossy().to_string();
    let host = format!("{}@{}", config.ssh.user, config.ssh.host);

    let mut command = Command::new("ssh");
    command.args([
        "-o",
        &format!("ControlPath={socket_str}"),
        "-O",
        "exit",
        &host,
    ]);

    let _ = command.output().await;
    let _ = std::fs::remove_file(&socket_path);
    tracing::info!("SSH ControlMaster fermé");
    Ok(())
}

/// Options SSH utilisant le `ControlMaster` existant
fn control_master_ssh_args(config: &AppConfig) -> Vec<String> {
    let socket_path = control_socket_path(config);
    vec![
        "-o".to_string(),
        format!("ControlPath={}", socket_path.to_string_lossy()),
        "-o".to_string(),
        "ControlMaster=no".to_string(),
    ]
}

/// Commande SSH pour rsync utilisant le `ControlMaster`
fn rsync_ssh_command(config: &AppConfig) -> String {
    let socket_path = control_socket_path(config);
    format!(
        "ssh -o ControlPath={} -o ControlMaster=no",
        socket_path.to_string_lossy()
    )
}

/// Exécute une commande SSH via le `ControlMaster`
pub async fn execute(config: &AppConfig, cmd: &str) -> Result<String, String> {
    let host = format!("{}@{}", config.ssh.user, config.ssh.host);

    let mut command = Command::new("ssh");
    for arg in control_master_ssh_args(config) {
        command.arg(&arg);
    }
    command.args([&host, cmd]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| format!("Erreur SSH: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Commande SSH échouée: {stderr}"))
    }
}

/// Exécute une commande SSH sans vérifier le code retour
pub async fn execute_ignore_status(config: &AppConfig, cmd: &str) -> Result<String, String> {
    let host = format!("{}@{}", config.ssh.user, config.ssh.host);

    let mut command = Command::new("ssh");
    for arg in control_master_ssh_args(config) {
        command.arg(&arg);
    }
    command.args([&host, cmd]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| format!("Erreur SSH: {e}"))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Teste la connexion SSH
pub async fn test_connection(config: &AppConfig) -> Result<bool, String> {
    Ok(execute(config, "echo ok")
        .await
        .is_ok_and(|output| output.trim() == "ok"))
}

/// Rsync dry-run pour compter les fichiers modifiés (code + dépendances)
pub async fn rsync_dry_run(config: &AppConfig) -> Result<Vec<String>, String> {
    let ssh_cmd = rsync_ssh_command(config);
    let local_path = format!("{}/", config.paths.local_code.to_string_lossy());
    let mut all_files: Vec<String> = Vec::new();

    // 1. Check pyproject.toml et uv.lock
    let remote_base = format!(
        "{}@{}:{}/",
        config.ssh.user, config.ssh.host, config.paths.remote_base
    );

    let mut deps_command = Command::new("rsync");
    deps_command.args([
        "-avzn", // dry-run
        "--include=pyproject.toml",
        "--include=uv.lock",
        "--exclude=*",
        "--out-format=%n",
        "-e",
        &ssh_cmd,
        &local_path,
        &remote_base,
    ]);

    deps_command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let deps_output = deps_command
        .output()
        .await
        .map_err(|e| format!("Erreur rsync dry-run dépendances: {e}"))?;

    let deps_stdout = String::from_utf8_lossy(&deps_output.stdout);
    for line in deps_stdout.lines() {
        if (line == "pyproject.toml" || line == "uv.lock") && !line.is_empty() {
            all_files.push(line.to_string());
        }
    }

    // 2. Check 2_SRC/
    let remote_code = format!(
        "{}@{}:{}/",
        config.ssh.user,
        config.ssh.host,
        config.remote_code_path()
    );

    let mut code_command = Command::new("rsync");
    code_command.args([
        "-avzn", // dry-run
        "--delete",
        // Sync uniquement 2_SRC/ avec ses fichiers .py
        "--include=2_SRC/",
        "--include=2_SRC/**",
        "--exclude=__pycache__/",
        "--exclude=*.pyc",
        "--exclude=*",     // Exclure tout le reste
        "--out-format=%n", // Format: juste le nom du fichier
        "-e",
        &ssh_cmd,
        &local_path,
        &remote_code,
    ]);

    code_command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let code_output = code_command
        .output()
        .await
        .map_err(|e| format!("Erreur rsync dry-run code: {e}"))?;

    let code_stdout = String::from_utf8_lossy(&code_output.stdout);

    // Filtrer pour garder seulement les fichiers .py modifiés
    for line in code_stdout.lines() {
        if !line.is_empty()
            && std::path::Path::new(line)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
        {
            all_files.push(line.to_string());
        }
    }

    Ok(all_files)
}

/// Rsync local vers serveur (code + dépendances)
pub async fn rsync_to_server(config: &AppConfig) -> Result<(), String> {
    let ssh_cmd = rsync_ssh_command(config);
    let local_path = format!("{}/", config.paths.local_code.to_string_lossy());

    // 1. Sync pyproject.toml et uv.lock vers ~/benchmarks/
    let remote_base = format!(
        "{}@{}:{}/",
        config.ssh.user, config.ssh.host, config.paths.remote_base
    );

    tracing::info!("Syncing pyproject.toml and uv.lock to {}", remote_base);

    let mut deps_command = Command::new("rsync");
    deps_command.args([
        "-avz",
        "--include=pyproject.toml",
        "--include=uv.lock",
        "--exclude=*",
        "-e",
        &ssh_cmd,
        &local_path,
        &remote_base,
    ]);

    deps_command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let deps_output = deps_command
        .output()
        .await
        .map_err(|e| format!("Erreur rsync dépendances: {e}"))?;

    if !deps_output.status.success() {
        let stderr = String::from_utf8_lossy(&deps_output.stderr);
        return Err(format!("rsync dépendances échoué: {stderr}"));
    }

    // 2. Sync 2_SRC/ vers ~/benchmarks/code/
    let remote_code = format!(
        "{}@{}:{}/",
        config.ssh.user,
        config.ssh.host,
        config.remote_code_path()
    );

    tracing::info!("Syncing 2_SRC/ to {}", remote_code);

    let mut code_command = Command::new("rsync");
    code_command.args([
        "-avz",
        "--delete",
        // Sync uniquement 2_SRC/ avec ses fichiers .py
        "--include=2_SRC/",
        "--include=2_SRC/**",
        "--exclude=__pycache__/",
        "--exclude=*.pyc",
        "--exclude=*", // Exclure tout le reste
        "-e",
        &ssh_cmd,
        &local_path,
        &remote_code,
    ]);

    code_command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let code_output = code_command
        .output()
        .await
        .map_err(|e| format!("Erreur rsync code: {e}"))?;

    if code_output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&code_output.stderr);
        Err(format!("rsync code échoué: {stderr}"))
    }
}

/// Rsync serveur vers local (résultats)
pub async fn rsync_from_server(
    config: &AppConfig,
    remote_path: &str,
    local_path: &str,
) -> Result<(), String> {
    let ssh_cmd = rsync_ssh_command(config);
    let remote_src = format!("{}@{}:{}", config.ssh.user, config.ssh.host, remote_path);

    let mut command = Command::new("rsync");
    command.args(["-avz", "-e", &ssh_cmd, &remote_src, local_path]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| format!("Erreur rsync: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("rsync échoué: {stderr}"))
    }
}

/// Vérifie si une session tmux existe
pub async fn tmux_session_exists(config: &AppConfig, session_name: &str) -> Result<bool, String> {
    let cmd = format!("tmux has-session -t {session_name} 2>/dev/null && echo yes || echo no");
    let result = execute_ignore_status(config, &cmd).await?;
    Ok(result.trim() == "yes")
}

/// Crée une session tmux pour exécuter un job
pub async fn start_tmux_job(
    config: &AppConfig,
    job_id: i64,
    benchmark_name: &str,
) -> Result<(), String> {
    let session_name = format!("job_{job_id}");
    let jobs_path = config.remote_jobs_path();
    let log_file = format!("{jobs_path}/{job_id}.log");

    // Créer les répertoires si nécessaire
    let mkdir_cmd = format!(
        "mkdir -p {} {} {}",
        config.remote_code_path(),
        config.remote_jobs_path(),
        config.remote_results_path()
    );
    execute(config, &mkdir_cmd).await?;

    // Lancer le job dans tmux
    // On travaille depuis remote_base (où sont pyproject.toml et .venv)
    // Le code est dans remote_base/code/
    // Export des variables Gurobi + PYTHONUNBUFFERED pour logs temps réel
    let base_dir = &config.paths.remote_base;
    let uv_path = &config.tools.uv_path;

    // Build Gurobi env exports (only if configured)
    let gurobi_exports = if !config.gurobi.home.is_empty() {
        format!(
            r#"export GUROBI_HOME="{}"; export GRB_LICENSE_FILE="{}"; export PATH="$PATH:$GUROBI_HOME/bin"; export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$GUROBI_HOME/lib"; "#,
            config.gurobi.home, config.gurobi.license_file
        )
    } else {
        String::new()
    };

    let cmd = format!(
        r#"tmux new-session -d -s {session_name} 'exec > {log_file} 2>&1; export PYTHONUNBUFFERED=1; {gurobi_exports}cd {base_dir} && echo "=== Starting job ===" && echo "Working directory: $(pwd)" && echo "=== uv sync ===" && {uv_path} sync && echo "=== Running benchmark ===" && {uv_path} run python code/2_SRC/benchmarks/{benchmark_name} ; echo "=== Job finished with code: $? ==="'"#
    );
    execute(config, &cmd).await?;

    Ok(())
}

/// Envoie Ctrl-C à une session tmux (stop propre)
pub async fn stop_tmux_job(config: &AppConfig, job_id: i64) -> Result<(), String> {
    let session_name = format!("job_{job_id}");
    let cmd = format!("tmux send-keys -t {session_name} C-c");
    execute_ignore_status(config, &cmd).await?;
    Ok(())
}

/// Kill une session tmux (arrêt forcé)
pub async fn kill_tmux_job(config: &AppConfig, job_id: i64) -> Result<(), String> {
    let session_name = format!("job_{job_id}");
    let cmd = format!("tmux kill-session -t {session_name}");
    execute_ignore_status(config, &cmd).await?;
    Ok(())
}

/// Récupère les dernières lignes du log d'un job
pub async fn get_job_logs(config: &AppConfig, job_id: i64, lines: u32) -> Result<String, String> {
    let jobs_path = config.remote_jobs_path();
    let log_file = format!("{jobs_path}/{job_id}.log");
    let cmd = format!("tail -n {lines} {log_file} 2>/dev/null || echo ''");
    execute_ignore_status(config, &cmd).await
}
