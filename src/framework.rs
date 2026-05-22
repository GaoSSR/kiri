use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub fn detect_framework(project_root: &Path) -> Option<&'static str> {
    let package_json = project_root.join("package.json");
    if package_json.exists() {
        if let Ok(contents) = fs::read_to_string(package_json) {
            if let Ok(package) = serde_json::from_str::<Value>(&contents) {
                if let Some(framework) = detect_framework_from_package(&package) {
                    return Some(framework);
                }
            }
        }
    }

    detect_framework_from_config(project_root)
}

pub fn detect_framework_from_command(command: &str, process_name: &str) -> Option<&'static str> {
    if command.is_empty() {
        return detect_framework_from_name(process_name);
    }

    let command = command.to_ascii_lowercase();

    if command.contains("next") {
        Some("Next.js")
    } else if command.contains("vite") {
        Some("Vite")
    } else if command.contains("nuxt") {
        Some("Nuxt")
    } else if command.contains("angular") || command.contains("ng serve") {
        Some("Angular")
    } else if command.contains("webpack") {
        Some("Webpack")
    } else if command.contains("remix") {
        Some("Remix")
    } else if command.contains("astro") {
        Some("Astro")
    } else if command.contains("gatsby") {
        Some("Gatsby")
    } else if command.contains("flask") {
        Some("Flask")
    } else if command.contains("django") || command.contains("manage.py") {
        Some("Django")
    } else if command.contains("uvicorn") {
        Some("FastAPI")
    } else if command.contains("rails") {
        Some("Rails")
    } else if command.contains("cargo") || command.contains("rustc") {
        Some("Rust")
    } else {
        detect_framework_from_name(process_name)
    }
}

pub fn detect_framework_from_name(process_name: &str) -> Option<&'static str> {
    match process_name.to_ascii_lowercase().as_str() {
        "node" => Some("Node.js"),
        "python" | "python3" => Some("Python"),
        "ruby" => Some("Ruby"),
        "java" => Some("Java"),
        "go" => Some("Go"),
        _ => None,
    }
}

fn detect_framework_from_package(package: &Value) -> Option<&'static str> {
    let dependencies = package.get("dependencies").and_then(Value::as_object);
    let dev_dependencies = package.get("devDependencies").and_then(Value::as_object);

    if has_dependency(dependencies, dev_dependencies, "next") {
        Some("Next.js")
    } else if has_dependency(dependencies, dev_dependencies, "nuxt")
        || has_dependency(dependencies, dev_dependencies, "nuxt3")
    {
        Some("Nuxt")
    } else if has_dependency(dependencies, dev_dependencies, "@sveltejs/kit") {
        Some("SvelteKit")
    } else if has_dependency(dependencies, dev_dependencies, "svelte") {
        Some("Svelte")
    } else if has_dependency(dependencies, dev_dependencies, "@remix-run/react")
        || has_dependency(dependencies, dev_dependencies, "remix")
    {
        Some("Remix")
    } else if has_dependency(dependencies, dev_dependencies, "astro") {
        Some("Astro")
    } else if has_dependency(dependencies, dev_dependencies, "vite") {
        Some("Vite")
    } else if has_dependency(dependencies, dev_dependencies, "@angular/core") {
        Some("Angular")
    } else if has_dependency(dependencies, dev_dependencies, "vue") {
        Some("Vue")
    } else if has_dependency(dependencies, dev_dependencies, "react") {
        Some("React")
    } else if has_dependency(dependencies, dev_dependencies, "express") {
        Some("Express")
    } else if has_dependency(dependencies, dev_dependencies, "fastify") {
        Some("Fastify")
    } else if has_dependency(dependencies, dev_dependencies, "hono") {
        Some("Hono")
    } else if has_dependency(dependencies, dev_dependencies, "koa") {
        Some("Koa")
    } else if has_dependency(dependencies, dev_dependencies, "nestjs")
        || has_dependency(dependencies, dev_dependencies, "@nestjs/core")
    {
        Some("NestJS")
    } else if has_dependency(dependencies, dev_dependencies, "gatsby") {
        Some("Gatsby")
    } else if has_dependency(dependencies, dev_dependencies, "webpack-dev-server") {
        Some("Webpack")
    } else if has_dependency(dependencies, dev_dependencies, "esbuild") {
        Some("esbuild")
    } else if has_dependency(dependencies, dev_dependencies, "parcel") {
        Some("Parcel")
    } else {
        None
    }
}

fn has_dependency(
    dependencies: Option<&Map<String, Value>>,
    dev_dependencies: Option<&Map<String, Value>>,
    name: &str,
) -> bool {
    dependencies.is_some_and(|deps| deps.contains_key(name))
        || dev_dependencies.is_some_and(|deps| deps.contains_key(name))
}

fn detect_framework_from_config(project_root: &Path) -> Option<&'static str> {
    if project_root.join("vite.config.ts").exists() || project_root.join("vite.config.js").exists()
    {
        Some("Vite")
    } else if project_root.join("next.config.js").exists()
        || project_root.join("next.config.mjs").exists()
    {
        Some("Next.js")
    } else if project_root.join("angular.json").exists() {
        Some("Angular")
    } else if project_root.join("Cargo.toml").exists() {
        Some("Rust")
    } else if project_root.join("go.mod").exists() {
        Some("Go")
    } else if project_root.join("manage.py").exists() {
        Some("Django")
    } else if project_root.join("Gemfile").exists() {
        Some("Ruby")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_framework_from_command_keywords() {
        let cases = [
            ("node ./node_modules/.bin/next dev", "node", Some("Next.js")),
            ("pnpm vite --host 0.0.0.0", "node", Some("Vite")),
            ("python -m uvicorn app:app", "python", Some("FastAPI")),
            ("python manage.py runserver", "python", Some("Django")),
            ("cargo run --bin api", "cargo", Some("Rust")),
        ];

        for (command, process_name, expected) in cases {
            assert_eq!(
                detect_framework_from_command(command, process_name),
                expected
            );
        }
    }

    #[test]
    fn command_detection_falls_back_to_process_name_or_none() {
        assert_eq!(
            detect_framework_from_command("/usr/local/bin/node server.js", "node"),
            Some("Node.js")
        );
        assert_eq!(
            detect_framework_from_command("/Applications/Slack.app/Contents/MacOS/Slack", "Slack"),
            None
        );
    }

    #[test]
    fn detects_framework_from_process_name() {
        let cases = [
            ("node", Some("Node.js")),
            ("python3", Some("Python")),
            ("java", Some("Java")),
            ("go", Some("Go")),
            ("unknown", None),
        ];

        for (process_name, expected) in cases {
            assert_eq!(detect_framework_from_name(process_name), expected);
        }
    }

    #[test]
    fn detects_framework_from_package_dependencies() {
        let root = unique_temp_dir("devports-framework-deps");
        fs::create_dir_all(&root).unwrap();

        let cases = [
            (r#"{"dependencies":{"next":"latest"}}"#, Some("Next.js")),
            (r#"{"dependencies":{"vite":"latest"}}"#, Some("Vite")),
            (r#"{"dependencies":{"react":"latest"}}"#, Some("React")),
            (r#"{"dependencies":{"express":"latest"}}"#, Some("Express")),
        ];

        for (idx, (package_json, expected)) in cases.into_iter().enumerate() {
            let project = root.join(format!("case-{idx}"));
            fs::create_dir_all(&project).unwrap();
            fs::write(project.join("package.json"), package_json).unwrap();

            assert_eq!(detect_framework(&project), expected);
        }
    }

    #[test]
    fn detects_framework_from_package_dev_dependencies() {
        let root = unique_temp_dir("devports-framework-dev-deps");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"devDependencies":{"vite":"latest"}}"#,
        )
        .unwrap();

        assert_eq!(detect_framework(&root), Some("Vite"));
    }

    #[test]
    fn malformed_package_json_returns_none_without_panicking() {
        let root = unique_temp_dir("devports-framework-malformed");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("package.json"), "{not-json").unwrap();

        assert_eq!(detect_framework(&root), None);
    }

    #[test]
    fn detects_framework_from_config_files() {
        let cases = [
            ("vite.config.ts", "Vite"),
            ("next.config.mjs", "Next.js"),
            ("Cargo.toml", "Rust"),
            ("go.mod", "Go"),
            ("manage.py", "Django"),
        ];

        for (marker, expected) in cases {
            let root = unique_temp_dir(&format!("devports-framework-{marker}"));
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join(marker), "").unwrap();

            assert_eq!(detect_framework(&root), Some(expected));
        }
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{unique}"))
    }
}
