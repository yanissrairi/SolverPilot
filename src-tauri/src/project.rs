//! Module de gestion des projets Python avec uv
//!
//! Chaque projet a son propre environnement Python géré par uv,
//! stocké dans `SolverPilot/projects/<nom>/`

use std::path::PathBuf;
use tokio::process::Command;

use crate::config::AppConfig;

// =============================================================================
// Paths
// =============================================================================

/// Retourne le dossier des projets (SolverPilot/projects/)
pub fn projects_dir() -> PathBuf {
    AppConfig::project_root().join("projects")
}

/// Retourne le chemin d'un projet spécifique
pub fn project_path(name: &str) -> PathBuf {
    projects_dir().join(name)
}

/// Retourne le chemin du pyproject.toml d'un projet
pub fn pyproject_path(name: &str) -> PathBuf {
    project_path(name).join("pyproject.toml")
}

// =============================================================================
// Project Lifecycle
// =============================================================================

/// Crée un nouveau projet avec `uv init`
pub async fn create_project(name: &str, python_version: &str, uv_path: &str) -> Result<(), String> {
    let project_dir = project_path(name);

    // Créer le dossier
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Erreur création dossier projet: {e}"))?;

    // Exécuter uv init avec la version Python spécifiée
    let uv = shellexpand::tilde(uv_path).to_string();
    let output = Command::new(&uv)
        .args(["init", "--python", python_version])
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv init: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Nettoyer le dossier en cas d'échec
        let _ = std::fs::remove_dir_all(&project_dir);
        return Err(format!("uv init a échoué: {stderr}"));
    }

    Ok(())
}

/// Supprime le dossier d'un projet
pub fn delete_project_dir(name: &str) -> Result<(), String> {
    let project_dir = project_path(name);
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir)
            .map_err(|e| format!("Erreur suppression dossier projet: {e}"))?;
    }
    Ok(())
}

// =============================================================================
// Python Version Management
// =============================================================================

/// Change la version Python d'un projet (met à jour pyproject.toml + uv python pin + uv lock)
pub async fn set_python_version(name: &str, version: &str, uv_path: &str) -> Result<(), String> {
    let project_dir = project_path(name);
    let uv = shellexpand::tilde(uv_path).to_string();

    // 1. Update requires-python in pyproject.toml (allows downgrade)
    let pyproject_path = project_dir.join("pyproject.toml");
    let content = std::fs::read_to_string(&pyproject_path)
        .map_err(|e| format!("Erreur lecture pyproject.toml: {e}"))?;

    let re = regex::Regex::new(r#"requires-python\s*=\s*">=[\d.]+""#)
        .map_err(|e| format!("Erreur regex: {e}"))?;
    let new_content = re.replace(&content, format!(r#"requires-python = ">={version}""#));

    std::fs::write(&pyproject_path, new_content.as_bytes())
        .map_err(|e| format!("Erreur écriture pyproject.toml: {e}"))?;

    // 2. Run uv python pin
    let output = Command::new(&uv)
        .args(["python", "pin", version])
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv python pin: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("uv python pin a échoué: {stderr}"));
    }

    // 3. Regenerate lockfile with new Python constraint
    let lock_output = Command::new(&uv)
        .args(["lock"])
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv lock: {e}"))?;

    if !lock_output.status.success() {
        let stderr = String::from_utf8_lossy(&lock_output.stderr);
        return Err(format!("uv lock a échoué: {stderr}"));
    }

    Ok(())
}

/// Liste les versions Python disponibles via `uv python list`
pub async fn list_python_versions(uv_path: &str) -> Result<Vec<String>, String> {
    let uv = shellexpand::tilde(uv_path).to_string();

    let output = Command::new(&uv)
        .args(["python", "list", "--all-versions"])
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv python list: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parser la sortie de uv python list
    // Format: "cpython-3.12.0-linux-x86_64-gnu  ..."
    let versions: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("cpython-") {
                // Extraire la version: cpython-3.12.0-... -> 3.12.0
                let parts: Vec<&str> = trimmed.split('-').collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string());
                }
            }
            None
        })
        .collect::<std::collections::HashSet<_>>() // Dédupliquer
        .into_iter()
        .collect();

    Ok(versions)
}

// =============================================================================
// Dependency Management
// =============================================================================

/// Ajoute une dépendance avec `uv add`
pub async fn add_dependency(name: &str, package: &str, uv_path: &str) -> Result<String, String> {
    let project_dir = project_path(name);
    let uv = shellexpand::tilde(uv_path).to_string();

    let output = Command::new(&uv)
        .args(["add", package])
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv add: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("✓ {package} ajouté\n{stdout}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("uv add a échoué: {stderr}"))
    }
}

/// Supprime une dépendance avec `uv remove`
pub async fn remove_dependency(name: &str, package: &str, uv_path: &str) -> Result<String, String> {
    let project_dir = project_path(name);
    let uv = shellexpand::tilde(uv_path).to_string();

    let output = Command::new(&uv)
        .args(["remove", package])
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv remove: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("✓ {package} supprimé\n{stdout}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("uv remove a échoué: {stderr}"))
    }
}

/// Met à jour toutes les dépendances avec `uv lock --upgrade`
pub async fn update_dependencies(name: &str, uv_path: &str) -> Result<String, String> {
    let project_dir = project_path(name);
    let uv = shellexpand::tilde(uv_path).to_string();

    let output = Command::new(&uv)
        .args(["lock", "--upgrade"])
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv lock --upgrade: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("✓ Dépendances mises à jour\n{stdout}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("uv lock --upgrade a échoué: {stderr}"))
    }
}

/// Synchronise l'environnement avec `uv sync`
pub async fn sync_environment(name: &str, uv_path: &str) -> Result<String, String> {
    let project_dir = project_path(name);
    let uv = shellexpand::tilde(uv_path).to_string();

    let output = Command::new(&uv)
        .arg("sync")
        .current_dir(&project_dir)
        .output()
        .await
        .map_err(|e| format!("Erreur exécution uv sync: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("✓ Environnement synchronisé\n{stdout}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("uv sync a échoué: {stderr}"))
    }
}

/// Lit les dépendances depuis pyproject.toml
pub fn read_project_dependencies(name: &str) -> Result<Vec<String>, String> {
    let pyproject = pyproject_path(name);

    let content = std::fs::read_to_string(&pyproject)
        .map_err(|e| format!("Erreur lecture pyproject.toml: {e}"))?;

    let value: toml::Value = content
        .parse()
        .map_err(|e| format!("Erreur parsing pyproject.toml: {e}"))?;

    // Format PEP 621: [project] dependencies = [...]
    let deps = value
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.as_str().map(|s| {
                        // Extraire le nom du package (avant les contraintes de version)
                        // Ex: "numpy>=1.0" -> "numpy"
                        s.split(['>', '<', '=', '!', '[', ';', ' '])
                            .next()
                            .unwrap_or(s)
                            .to_string()
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(deps)
}

/// Vérifie si le dossier projet existe
pub fn project_exists(name: &str) -> bool {
    project_path(name).exists()
}
