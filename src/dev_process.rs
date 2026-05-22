pub fn is_developer_process(process_name: &str, command: &str) -> bool {
    let name = process_name.to_ascii_lowercase();
    let command = command.to_ascii_lowercase();

    if SYSTEM_PROCESS_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return false;
    }

    DEV_PROCESS_NAMES.contains(&name.as_str())
        || name.starts_with("com.docke")
        || name == "docker-sandbox"
        || command_has_developer_indicator(&command)
}

fn command_has_developer_indicator(command: &str) -> bool {
    if command.is_empty() {
        return false;
    }

    if command.contains("manage.py") || command.contains("ng serve") {
        return true;
    }

    COMMAND_INDICATORS
        .iter()
        .any(|indicator| contains_word(command, indicator))
}

fn contains_word(value: &str, word: &str) -> bool {
    value
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '.')
        .any(|part| part == word)
}

const SYSTEM_PROCESS_PREFIXES: &[&str] = &[
    "spotify",
    "raycast",
    "tableplus",
    "postman",
    "linear",
    "cursor",
    "controlce",
    "rapportd",
    "slack",
    "discord",
    "firefox",
    "chrome",
    "google",
    "safari",
    "figma",
    "notion",
    "zoom",
    "teams",
    "code",
    "iterm2",
    "warp",
    "arc",
    "loginwindow",
    "windowserver",
    "systemuise",
    "kernel_task",
    "launchd",
    "mdworker",
    "mds_stores",
    "cfprefsd",
    "coreaudio",
    "airportd",
    "bluetoothd",
    "sharingd",
    "systemd",
    "snapd",
    "networkmanager",
    "gdm",
    "sshd",
    "cron",
    "dbus-daemon",
    "svchost",
    "csrss",
    "lsass",
    "services",
    "explorer",
];

const DEV_PROCESS_NAMES: &[&str] = &[
    "node",
    "python",
    "python3",
    "ruby",
    "java",
    "go",
    "cargo",
    "deno",
    "bun",
    "php",
    "uvicorn",
    "gunicorn",
    "flask",
    "rails",
    "npm",
    "npx",
    "yarn",
    "pnpm",
    "tsc",
    "tsx",
    "esbuild",
    "rollup",
    "turbo",
    "nx",
    "jest",
    "vitest",
    "mocha",
    "pytest",
    "cypress",
    "playwright",
    "rustc",
    "dotnet",
    "gradle",
    "mvn",
    "mix",
    "elixir",
    "docker",
];

const COMMAND_INDICATORS: &[&str] = &[
    "node", "next", "vite", "nuxt", "webpack", "remix", "astro", "gulp", "gatsby", "flask",
    "django", "uvicorn", "rails", "cargo",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_common_developer_processes_by_name() {
        assert!(is_developer_process("node", ""));
        assert!(is_developer_process("python3", ""));
        assert!(is_developer_process("java", ""));
        assert!(is_developer_process("docker", ""));
        assert!(is_developer_process("com.docker.backend", ""));
    }

    #[test]
    fn excludes_common_desktop_and_system_processes() {
        assert!(!is_developer_process("Chrome", ""));
        assert!(!is_developer_process("Slack", ""));
        assert!(!is_developer_process("systemd", ""));
    }

    #[test]
    fn identifies_developer_processes_from_command_indicators() {
        assert!(is_developer_process(
            "bash",
            "/bin/bash -lc npm run vite -- --host 0.0.0.0"
        ));
        assert!(is_developer_process(
            "python",
            "python manage.py runserver 0.0.0.0:8000"
        ));
    }
}
