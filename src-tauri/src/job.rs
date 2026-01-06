use std::path::Path;

use crate::state::Benchmark;

/// Parse la progression depuis les logs
/// Format attendu: [12/22] Config: ...
pub fn parse_progress(logs: &str) -> Option<(u32, u32)> {
    let mut last_match: Option<(u32, u32)> = None;

    for line in logs.lines() {
        if let Some(start) = line.find('[') {
            if let Some(end) = line[start..].find(']') {
                let bracket_content = &line[start + 1..start + end];
                if let Some(slash) = bracket_content.find('/') {
                    let current_str = &bracket_content[..slash];
                    let total_str = &bracket_content[slash + 1..];

                    if let (Ok(c), Ok(t)) = (current_str.parse::<u32>(), total_str.parse::<u32>()) {
                        last_match = Some((c, t));
                    }
                }
            }
        }
    }
    last_match
}

/// Détecte si le job est terminé en analysant les logs
pub fn detect_job_finished(logs: &str) -> bool {
    let finish_patterns = [
        "RÉSUMÉ BENCHMARK",
        "Résultats dans:",
        "benchmark_results.csv",
        "Total:",
    ];

    finish_patterns.iter().any(|p| logs.contains(p))
}

/// Détecte une erreur dans les logs
pub fn detect_job_error(logs: &str) -> Option<String> {
    if logs.contains("Traceback") || logs.contains("Error:") || logs.contains("Exception:") {
        let lines: Vec<&str> = logs.lines().collect();
        Some(
            lines
                .iter()
                .rev()
                .take(5)
                .copied()
                .rev()
                .collect::<Vec<_>>()
                .join("\n"),
        )
    } else {
        None
    }
}

/// Scanne les fichiers benchmark_*.py dans un répertoire
pub fn scan_benchmarks(benchmarks_dir: &Path) -> Vec<Benchmark> {
    let mut benchmarks = Vec::new();

    if let Ok(entries) = std::fs::read_dir(benchmarks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("benchmark_")
                    && Path::new(name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
                {
                    benchmarks.push(Benchmark {
                        name: name.to_string(),
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    benchmarks.sort_by(|a, b| a.name.cmp(&b.name));
    benchmarks
}

/// Formate le temps écoulé
pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{mins}m {remaining_secs}s")
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{hours}h {mins}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress() {
        let logs = r#"
[1/22] Config: N5_FF1_Tauto
  Grille: 5x5
[2/22] Config: N5_FF2_Tauto
  Résolu en 1.2s
[3/22] Config: N7_FF1_Tauto
"#;
        assert_eq!(parse_progress(logs), Some((3, 22)));
    }

    #[test]
    fn test_detect_finished() {
        let logs = "RÉSUMÉ BENCHMARK\nTerminé avec succès";
        assert!(detect_job_finished(logs));

        let logs2 = "[5/10] En cours...";
        assert!(!detect_job_finished(logs2));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(125), "2m 5s");
        assert_eq!(format_duration(3725), "1h 2m");
    }
}
