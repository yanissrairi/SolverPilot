//! Analyse des dépendances Python avec tree-sitter
//!
//! Ce module permet d'analyser les imports d'un fichier Python et de construire
//! un arbre de dépendances récursif pour les fichiers locaux.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

/// Résultat de l'analyse des dépendances d'un benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// Fichier racine analysé
    pub root: String,
    /// Fichiers Python locaux (récursifs)
    pub local_files: Vec<LocalDependency>,
    /// Packages externes (pip)
    pub external_packages: Vec<ExternalPackage>,
}

/// Une dépendance locale (fichier .py du projet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDependency {
    /// Nom du module (ex: "solvers.firefighter")
    pub module_name: String,
    /// Chemin absolu du fichier
    pub file_path: String,
    /// Le fichier existe-t-il?
    pub exists: bool,
    /// Dépendances enfants (récursives)
    pub children: Vec<LocalDependency>,
}

/// Un package externe (pip/pypi)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPackage {
    /// Nom du package (ex: "numpy")
    pub name: String,
    /// Est-il déclaré dans pyproject.toml?
    pub in_pyproject: bool,
}

/// Information extraite d'un fichier Python
#[derive(Debug, Default)]
struct PythonFileInfo {
    /// Imports simples: `import X` ou `import X.Y`
    imports: Vec<String>,
    /// From imports: `from X import Y` -> on garde juste X
    from_imports: Vec<String>,
    /// Modifications de sys.path (chemins à ajouter)
    sys_path_additions: Vec<String>,
}

/// Contexte pour l'analyse récursive des dépendances
struct AnalysisContext<'a> {
    /// Chemins de recherche pour les imports locaux
    search_paths: Vec<PathBuf>,
    /// Fichiers visités (détection de cycles)
    visited: HashSet<PathBuf>,
    /// Fichiers ajoutés globalement (déduplication)
    added_globally: HashSet<PathBuf>,
    /// Cache des dépendances enfants
    cache: HashMap<PathBuf, Vec<LocalDependency>>,
    /// Racine du code local
    local_code_root: &'a Path,
    /// Packages externes trouvés
    external_set: HashSet<String>,
}

/// Analyseur de dépendances Python
pub struct PythonAnalyzer {
    parser: Parser,
    import_query: Query,
}

impl PythonAnalyzer {
    /// Crée un nouvel analyseur
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;

        parser
            .set_language(&language.into())
            .map_err(|e| format!("Erreur initialisation tree-sitter: {e}"))?;

        // Query pour les imports
        // Note: `import X as Y` crée un aliased_import, pas un dotted_name direct
        let import_query = Query::new(
            &language.into(),
            r"
            (import_statement
                name: (dotted_name) @import)
            (import_statement
                name: (aliased_import
                    name: (dotted_name) @import))
            (import_from_statement
                module_name: (dotted_name) @from_import)
            (import_from_statement
                module_name: (relative_import) @relative_import)
            ",
        )
        .map_err(|e| format!("Erreur query imports: {e}"))?;

        Ok(Self {
            parser,
            import_query,
        })
    }

    /// Parse un fichier Python et extrait les informations
    fn parse_file(&mut self, source: &str, file_path: &Path) -> PythonFileInfo {
        let mut info = PythonFileInfo::default();

        let Some(tree) = self.parser.parse(source, None) else {
            return info;
        };

        let root_node = tree.root_node();
        let source_bytes = source.as_bytes();

        // Extraire les imports
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.import_query, root_node, source_bytes);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let text = capture.node.utf8_text(source_bytes).unwrap_or("");
                let capture_name = self.import_query.capture_names()[capture.index as usize];

                match capture_name {
                    "import" => {
                        info.imports.push(text.to_string());
                    }
                    "from_import" => {
                        info.from_imports.push(text.to_string());
                    }
                    "relative_import" => {
                        // Relative imports like "from . import X" - we'll handle these specially
                        if let Some(resolved) = Self::resolve_relative_import(text, file_path) {
                            info.from_imports.push(resolved);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Extraire les modifications sys.path
        info.sys_path_additions = Self::extract_sys_path_additions(source, file_path);

        info
    }

    /// Résout un import relatif vers un nom de module
    fn resolve_relative_import(relative: &str, file_path: &Path) -> Option<String> {
        // Count leading dots
        let dots = relative.chars().take_while(|c| *c == '.').count();
        let remainder = &relative[dots..];

        // Get parent directory (dots - 1 levels up from file's directory)
        let mut current_dir = file_path.parent()?;
        for _ in 1..dots {
            current_dir = current_dir.parent()?;
        }

        // Try to construct a module name from the directory structure
        if remainder.is_empty() {
            current_dir.file_name()?.to_str().map(String::from)
        } else {
            Some(remainder.to_string())
        }
    }

    /// Extrait les chemins ajoutés via sys.path.insert/append
    /// avec évaluation statique des variables de type os.path.dirname
    fn extract_sys_path_additions(source: &str, file_path: &Path) -> Vec<String> {
        let mut additions = Vec::new();
        let file_dir = file_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf();

        // Tracking des variables de chemin (évaluation statique simple)
        // Ex: _current_dir = os.path.dirname(__file__) → file_dir
        //     _src_dir = os.path.dirname(_current_dir) → file_dir.parent()
        let mut path_vars: HashMap<String, PathBuf> = HashMap::new();

        for line in source.lines() {
            let line = line.trim();

            // Pattern: VAR = os.path.dirname(os.path.abspath(__file__))
            // ou: VAR = os.path.dirname(__file__)
            if line.contains("os.path.dirname") && line.contains("__file__") {
                if let Some((var_name, _)) = line.split_once('=') {
                    let var_name = var_name.trim().to_string();
                    path_vars.insert(var_name, file_dir.clone());
                }
            }
            // Pattern: VAR = os.path.dirname(OTHER_VAR)
            else if line.contains("os.path.dirname") {
                if let Some((var_name, rhs)) = line.split_once('=') {
                    let var_name = var_name.trim().to_string();
                    // Chercher la variable référencée
                    for (known_var, known_path) in &path_vars {
                        if rhs.contains(known_var) {
                            if let Some(parent) = known_path.parent() {
                                path_vars.insert(var_name.clone(), parent.to_path_buf());
                                break;
                            }
                        }
                    }
                }
            }

            // Pattern: sys.path.insert(0, os.path.join(VAR, 'folder'))
            // ou: sys.path.append(os.path.join(VAR, 'folder'))
            if line.contains("sys.path") && line.contains("os.path.join") {
                // Extraire la variable de base et le sous-dossier
                let mut base_path: Option<PathBuf> = None;
                let mut subfolder: Option<String> = None;

                // Chercher quelle variable est utilisée
                for (var_name, var_path) in &path_vars {
                    if line.contains(var_name) {
                        base_path = Some(var_path.clone());
                        break;
                    }
                }

                // Extraire le string entre quotes (le sous-dossier)
                let mut in_quote = false;
                let mut quote_char = ' ';
                let mut current = String::new();

                for c in line.chars() {
                    if !in_quote && (c == '"' || c == '\'') {
                        in_quote = true;
                        quote_char = c;
                    } else if in_quote && c == quote_char {
                        in_quote = false;
                        if !current.is_empty() {
                            subfolder = Some(current.clone());
                        }
                        current.clear();
                    } else if in_quote {
                        current.push(c);
                    }
                }

                // Construire le chemin final
                if let (Some(base), Some(sub)) = (base_path, subfolder) {
                    let full_path = base.join(&sub);
                    if full_path.exists() {
                        additions.push(full_path.to_string_lossy().to_string());
                    }
                }
            }
            // Pattern simple: sys.path.insert/append avec juste un string
            else if line.contains("sys.path") {
                let mut in_quote = false;
                let mut quote_char = ' ';
                let mut current = String::new();

                for c in line.chars() {
                    if !in_quote && (c == '"' || c == '\'') {
                        in_quote = true;
                        quote_char = c;
                    } else if in_quote && c == quote_char {
                        in_quote = false;
                        if !current.is_empty() {
                            let path = file_dir.join(&current);
                            if path.exists() {
                                additions.push(path.to_string_lossy().to_string());
                            }
                        }
                        current.clear();
                    } else if in_quote {
                        current.push(c);
                    }
                }
            }
        }

        additions
    }

    /// Analyse les dépendances d'un fichier de façon récursive
    pub fn analyze(
        &mut self,
        benchmark_path: &Path,
        local_code_root: &Path,
        pyproject_path: Option<&Path>,
    ) -> Result<DependencyAnalysis, String> {
        let source = std::fs::read_to_string(benchmark_path)
            .map_err(|e| format!("Erreur lecture {}: {e}", benchmark_path.display()))?;

        let info = self.parse_file(&source, benchmark_path);

        // Construire les chemins de recherche
        let mut search_paths = vec![
            benchmark_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
            local_code_root.to_path_buf(),
        ];

        // Ajouter les chemins de sys.path
        for path_str in &info.sys_path_additions {
            let path = if path_str.starts_with('/') {
                PathBuf::from(path_str)
            } else {
                local_code_root.join(path_str)
            };
            if path.exists() && !search_paths.contains(&path) {
                search_paths.push(path);
            }
        }

        // Collecter les dépendances déclarées dans pyproject.toml
        let declared_deps = pyproject_path.map(parse_pyproject_deps).unwrap_or_default();

        // Contexte d'analyse récursive
        let mut ctx = AnalysisContext {
            search_paths,
            visited: HashSet::new(),
            added_globally: HashSet::new(),
            cache: HashMap::new(),
            local_code_root,
            external_set: HashSet::new(),
        };

        let mut local_files = Vec::new();

        // Tous les imports du fichier racine
        let all_imports: Vec<_> = info
            .imports
            .iter()
            .chain(info.from_imports.iter())
            .cloned()
            .collect();

        for import in &all_imports {
            let module_name = import.split('.').next().unwrap_or(import);

            // Essayer de résoudre comme fichier local
            if let Some(local_path) = Self::resolve_local_import(import, &ctx.search_paths) {
                let canonical = local_path
                    .canonicalize()
                    .unwrap_or_else(|_| local_path.clone());

                // Déduplication : ne pas ajouter si déjà traité
                if ctx.added_globally.insert(canonical) {
                    let dep = self.analyze_local_dependency(import, &local_path, &mut ctx);
                    local_files.push(dep);
                }
            } else {
                // C'est un package externe
                ctx.external_set.insert(module_name.to_string());
            }
        }

        // Vérifier les packages externes contre pyproject.toml
        let external_packages: Vec<ExternalPackage> = ctx
            .external_set
            .into_iter()
            .filter(|name| !is_stdlib_module(name))
            .map(|name| {
                let in_pyproject = declared_deps.contains(&name.to_lowercase());
                ExternalPackage { name, in_pyproject }
            })
            .collect();

        Ok(DependencyAnalysis {
            root: benchmark_path.to_string_lossy().to_string(),
            local_files,
            external_packages,
        })
    }

    /// Résout un import vers un fichier local
    fn resolve_local_import(import: &str, search_paths: &[PathBuf]) -> Option<PathBuf> {
        // Convertir le nom de module en chemin relatif
        let relative_path = import.replace('.', "/");

        for search_path in search_paths {
            // Essayer comme fichier .py
            let py_file = search_path.join(format!("{relative_path}.py"));
            if py_file.exists() {
                return Some(py_file);
            }

            // Essayer comme package (__init__.py)
            let init_file = search_path.join(&relative_path).join("__init__.py");
            if init_file.exists() {
                return Some(init_file);
            }

            // Essayer juste le dossier (package sans __init__)
            let package_dir = search_path.join(&relative_path);
            if package_dir.is_dir() {
                // Chercher le premier .py dans le dossier
                if let Ok(entries) = std::fs::read_dir(&package_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "py") {
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    /// Analyse récursive d'une dépendance locale
    fn analyze_local_dependency(
        &mut self,
        module_name: &str,
        file_path: &Path,
        ctx: &mut AnalysisContext<'_>,
    ) -> LocalDependency {
        let canonical = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());

        // Éviter les cycles
        if ctx.visited.contains(&canonical) {
            return LocalDependency {
                module_name: module_name.to_string(),
                file_path: file_path.to_string_lossy().to_string(),
                exists: file_path.exists(),
                children: vec![],
            };
        }

        // Vérifier le cache
        if let Some(cached_children) = ctx.cache.get(&canonical) {
            return LocalDependency {
                module_name: module_name.to_string(),
                file_path: file_path.to_string_lossy().to_string(),
                exists: true,
                children: cached_children.clone(),
            };
        }

        ctx.visited.insert(canonical.clone());

        let mut children = Vec::new();

        if file_path.exists() {
            if let Ok(source) = std::fs::read_to_string(file_path) {
                let info = self.parse_file(&source, file_path);

                // Ajouter les sys.path additions locales
                let mut local_search_paths = ctx.search_paths.clone();
                for path_str in &info.sys_path_additions {
                    let path = if path_str.starts_with('/') {
                        PathBuf::from(path_str)
                    } else {
                        ctx.local_code_root.join(path_str)
                    };
                    if path.exists() && !local_search_paths.contains(&path) {
                        local_search_paths.push(path);
                    }
                }

                // Analyser les imports de ce fichier
                let all_imports: Vec<_> = info
                    .imports
                    .iter()
                    .chain(info.from_imports.iter())
                    .cloned()
                    .collect();

                // Sauvegarder et restaurer search_paths pour la récursion
                let original_search_paths =
                    std::mem::replace(&mut ctx.search_paths, local_search_paths);

                for import in &all_imports {
                    let import_module_name = import.split('.').next().unwrap_or(import);

                    if let Some(local_path) = Self::resolve_local_import(import, &ctx.search_paths)
                    {
                        let local_canonical = local_path
                            .canonicalize()
                            .unwrap_or_else(|_| local_path.clone());

                        // Déduplication globale : n'ajouter comme enfant que si pas déjà ajouté ailleurs
                        if ctx.added_globally.insert(local_canonical) {
                            let child = self.analyze_local_dependency(import, &local_path, ctx);
                            children.push(child);
                        }
                    } else {
                        // Package externe - collecter récursivement
                        ctx.external_set.insert(import_module_name.to_string());
                    }
                }

                // Restaurer les search_paths originaux
                ctx.search_paths = original_search_paths;
            }
        }

        // Mettre en cache
        ctx.cache.insert(canonical, children.clone());

        LocalDependency {
            module_name: module_name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            exists: file_path.exists(),
            children,
        }
    }
}

/// Parse les dépendances déclarées dans pyproject.toml
fn parse_pyproject_deps(path: &Path) -> HashSet<String> {
    let mut deps = HashSet::new();

    let Ok(content) = std::fs::read_to_string(path) else {
        return deps;
    };

    // Parse TOML
    let Ok(value) = content.parse::<toml::Value>() else {
        return deps;
    };

    // [project.dependencies] ou [tool.poetry.dependencies]
    if let Some(project_deps) = value
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for dep in project_deps {
            if let Some(dep_str) = dep.as_str() {
                // Format: "numpy>=1.0" ou "numpy"
                let name = dep_str
                    .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                if !name.is_empty() {
                    deps.insert(name);
                }
            }
        }
    }

    // [tool.uv.sources] ou [tool.poetry.dependencies]
    if let Some(poetry_deps) = value
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_table())
    {
        for name in poetry_deps.keys() {
            deps.insert(name.to_lowercase());
        }
    }

    deps
}

/// Vérifie si un module fait partie de la stdlib Python
fn is_stdlib_module(name: &str) -> bool {
    // Liste non exhaustive des modules stdlib les plus courants
    const STDLIB: &[&str] = &[
        "abc",
        "argparse",
        "ast",
        "asyncio",
        "base64",
        "collections",
        "contextlib",
        "copy",
        "csv",
        "dataclasses",
        "datetime",
        "decimal",
        "difflib",
        "enum",
        "functools",
        "gc",
        "glob",
        "hashlib",
        "heapq",
        "html",
        "http",
        "importlib",
        "inspect",
        "io",
        "itertools",
        "json",
        "logging",
        "math",
        "multiprocessing",
        "operator",
        "os",
        "pathlib",
        "platform",
        "pprint",
        "queue",
        "random",
        "re",
        "shutil",
        "signal",
        "socket",
        "sqlite3",
        "statistics",
        "string",
        "subprocess",
        "sys",
        "tempfile",
        "textwrap",
        "threading",
        "time",
        "traceback",
        "typing",
        "unittest",
        "urllib",
        "uuid",
        "warnings",
        "weakref",
        "xml",
        "zipfile",
    ];

    STDLIB.contains(&name)
}

impl DependencyAnalysis {
    /// Collecte tous les chemins de fichiers locaux (récursifs)
    pub fn collect_all_file_paths(&self) -> Vec<String> {
        let mut paths = vec![self.root.clone()];
        for dep in &self.local_files {
            Self::collect_paths_recursive(dep, &mut paths);
        }
        paths
    }

    fn collect_paths_recursive(dep: &LocalDependency, paths: &mut Vec<String>) {
        if dep.exists && !paths.contains(&dep.file_path) {
            paths.push(dep.file_path.clone());
        }
        for child in &dep.children {
            Self::collect_paths_recursive(child, paths);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_detection() {
        assert!(is_stdlib_module("os"));
        assert!(is_stdlib_module("sys"));
        assert!(is_stdlib_module("json"));
        assert!(!is_stdlib_module("numpy"));
        assert!(!is_stdlib_module("pandas"));
    }

    #[test]
    fn test_pyproject_parsing() {
        let content = r#"
[project]
name = "my-project"
dependencies = [
    "numpy>=1.20",
    "pandas",
    "gurobipy>=10.0",
]
"#;
        let temp_dir = std::env::temp_dir();
        let pyproject_path = temp_dir.join("test_pyproject.toml");
        std::fs::write(&pyproject_path, content).unwrap();

        let deps = parse_pyproject_deps(&pyproject_path);
        assert!(deps.contains("numpy"));
        assert!(deps.contains("pandas"));
        assert!(deps.contains("gurobipy"));

        std::fs::remove_file(pyproject_path).ok();
    }
}
