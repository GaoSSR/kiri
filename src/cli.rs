use crate::kill::run_kill_command;
use crate::logs::run_logs_command;
use crate::process::get_all_processes;
use crate::render::table::{
    render_port_detail_with_color_mode, render_port_table_with_color_mode,
    render_process_table_with_color_mode, ColorMode,
};
use crate::scanner::{get_listening_ports, get_port_details};

pub fn run_from_env() -> i32 {
    run(std::env::args().skip(1))
}

pub fn run<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    match parse_args(args) {
        Ok(Command::List {
            show_all,
            color_mode,
        }) => {
            let ports = get_listening_ports(show_all);
            print!(
                "{}",
                render_port_table_with_color_mode(&ports, !show_all, color_mode)
            );
            0
        }
        Ok(Command::PortDetails { port, color_mode }) => {
            match get_port_details(port) {
                Some(info) => print!("{}", render_port_detail_with_color_mode(&info, color_mode)),
                None => println!("No process found listening on :{port}."),
            }
            0
        }
        Ok(Command::Kill { args }) => {
            let outcome = run_kill_command(&args);
            print!("{}", outcome.output);
            outcome.exit_code
        }
        Ok(Command::Ps {
            show_all,
            color_mode,
        }) => {
            let processes = get_all_processes(show_all);
            print!(
                "{}",
                render_process_table_with_color_mode(&processes, !show_all, color_mode)
            );
            0
        }
        Ok(Command::Logs { args }) => {
            let outcome = run_logs_command(&args);
            print!("{}", outcome.output);
            outcome.exit_code
        }
        Ok(Command::Help) => {
            print_help();
            0
        }
        Err(message) => {
            eprintln!("{message}");
            eprintln!("Run `devports --help` for usage.");
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    List {
        show_all: bool,
        color_mode: ColorMode,
    },
    PortDetails {
        port: u16,
        color_mode: ColorMode,
    },
    Kill {
        args: Vec<String>,
    },
    Ps {
        show_all: bool,
        color_mode: ColorMode,
    },
    Logs {
        args: Vec<String>,
    },
    Help,
}

fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let (args, color_mode) = extract_color_mode(args)?;
    let (args, show_all) = extract_show_all(args);

    if args.is_empty() {
        return Ok(Command::List {
            show_all,
            color_mode,
        });
    }

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(Command::Help);
    }

    match args[0].as_str() {
        "kill" => {
            let kill_args = args.into_iter().skip(1).collect();
            Ok(Command::Kill { args: kill_args })
        }
        "ps" => {
            if args.len() > 1 {
                return Err(format!("Unknown arguments: {}", args[1..].join(" ")));
            }
            Ok(Command::Ps {
                show_all,
                color_mode,
            })
        }
        "logs" => {
            let logs_args = args.into_iter().skip(1).collect();
            Ok(Command::Logs { args: logs_args })
        }
        command => {
            if args.len() > 1 {
                return Err(format!("Unknown arguments: {}", args[1..].join(" ")));
            }
            parse_port(command).map(|port| Command::PortDetails { port, color_mode })
        }
    }
}

fn extract_color_mode(args: Vec<String>) -> Result<(Vec<String>, ColorMode), String> {
    let mut output = Vec::with_capacity(args.len());
    let mut color_mode = ColorMode::Auto;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "--color" {
            let Some(value) = args.get(index + 1) else {
                return Err("--color requires one of: auto, always, never.".to_string());
            };
            color_mode = parse_color_mode(value)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--color=") {
            color_mode = parse_color_mode(value)?;
            index += 1;
        } else {
            output.push(arg.clone());
            index += 1;
        }
    }

    Ok((output, color_mode))
}

fn extract_show_all(args: Vec<String>) -> (Vec<String>, bool) {
    let mut show_all = false;
    let mut output = Vec::with_capacity(args.len());

    for arg in args {
        if arg == "--all" || arg == "-a" {
            show_all = true;
        } else {
            output.push(arg);
        }
    }

    (output, show_all)
}

fn parse_color_mode(value: &str) -> Result<ColorMode, String> {
    match value {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        _ => Err(format!(
            "`{value}` is not a valid color mode. Use auto, always, or never."
        )),
    }
}

fn parse_port(value: &str) -> Result<u16, String> {
    let port = value
        .parse::<u32>()
        .map_err(|_| format!("`{value}` is not a valid port."))?;

    if port == 0 || port > u16::MAX as u32 {
        return Err(format!(
            "`{value}` is outside the valid port range 1-65535."
        ));
    }

    Ok(port as u16)
}

fn print_help() {
    println!("DevPorts - inspect local listening ports");
    println!();
    println!("Usage:");
    println!("  devports          Show developer listening ports");
    println!("  devports --all    Show all listening ports");
    println!("  devports <port>   Show port details (Phase 2)");
    println!("  devports ps       Show running developer processes");
    println!("  devports logs <port|pid> [-f|--follow] [--lines N] [--err]");
    println!("  devports kill [-f|--force] <port|pid|range> [...]");
    println!("  devports --color <auto|always|never>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_list_command() {
        assert_eq!(
            parse_args(Vec::<String>::new()),
            Ok(Command::List {
                show_all: false,
                color_mode: ColorMode::Auto,
            })
        );
    }

    #[test]
    fn parses_all_list_command() {
        assert_eq!(
            parse_args(["--all"]),
            Ok(Command::List {
                show_all: true,
                color_mode: ColorMode::Auto,
            })
        );
        assert_eq!(
            parse_args(["-a"]),
            Ok(Command::List {
                show_all: true,
                color_mode: ColorMode::Auto,
            })
        );
    }

    #[test]
    fn parses_color_mode_for_list_commands() {
        assert_eq!(
            parse_args(["--color", "always"]),
            Ok(Command::List {
                show_all: false,
                color_mode: ColorMode::Always,
            })
        );
        assert_eq!(
            parse_args(["--all", "--color", "never"]),
            Ok(Command::List {
                show_all: true,
                color_mode: ColorMode::Never,
            })
        );
        assert_eq!(
            parse_args(["--color=always", "--all"]),
            Ok(Command::List {
                show_all: true,
                color_mode: ColorMode::Always,
            })
        );
        assert_eq!(
            parse_args(["--color", "auto", "--all"]),
            Ok(Command::List {
                show_all: true,
                color_mode: ColorMode::Auto,
            })
        );
    }

    #[test]
    fn parses_ps_command_with_all_and_color() {
        assert_eq!(
            parse_args(["ps"]),
            Ok(Command::Ps {
                show_all: false,
                color_mode: ColorMode::Auto,
            })
        );
        assert_eq!(
            parse_args(["ps", "--all", "--color=never"]),
            Ok(Command::Ps {
                show_all: true,
                color_mode: ColorMode::Never,
            })
        );
        assert_eq!(
            parse_args(["--all", "ps"]),
            Ok(Command::Ps {
                show_all: true,
                color_mode: ColorMode::Auto,
            })
        );
    }

    #[test]
    fn parses_color_mode_for_port_details() {
        assert_eq!(
            parse_args(["--color", "always", "5173"]),
            Ok(Command::PortDetails {
                port: 5173,
                color_mode: ColorMode::Always,
            })
        );
    }

    #[test]
    fn rejects_invalid_color_mode() {
        assert_eq!(
            parse_args(["--color"]),
            Err("--color requires one of: auto, always, never.".to_string())
        );
        assert_eq!(
            parse_args(["--color", "sometimes"]),
            Err("`sometimes` is not a valid color mode. Use auto, always, or never.".to_string())
        );
    }

    #[test]
    fn parses_deferred_commands() {
        assert_eq!(
            parse_args(["3000"]),
            Ok(Command::PortDetails {
                port: 3000,
                color_mode: ColorMode::Auto,
            })
        );
        assert_eq!(
            parse_args(["kill", "3000"]),
            Ok(Command::Kill {
                args: vec!["3000".to_string()]
            })
        );
        assert_eq!(
            parse_args(["logs", "3000", "--lines", "5", "--err"]),
            Ok(Command::Logs {
                args: vec![
                    "3000".to_string(),
                    "--lines".to_string(),
                    "5".to_string(),
                    "--err".to_string()
                ]
            })
        );
    }

    #[test]
    fn keeps_kill_arguments_for_kill_module() {
        assert_eq!(
            parse_args(["kill", "-f", "3000", "3001-3002"]),
            Ok(Command::Kill {
                args: vec![
                    "-f".to_string(),
                    "3000".to_string(),
                    "3001-3002".to_string()
                ]
            })
        );
    }
}
