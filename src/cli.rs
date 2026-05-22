use crate::clean::run_clean_command;
use crate::kill::run_kill_command;
use crate::logs::run_logs_command;
use crate::process::get_all_processes;
use crate::render::table::{render_port_detail, render_port_table, render_process_table};
use crate::scanner::{get_listening_ports, get_port_details};
use crate::watch::run_watch_command;

pub fn run_from_env() -> i32 {
    run(std::env::args().skip(1))
}

pub fn run<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    match parse_args(args) {
        Ok(Command::List { show_all }) => {
            let ports = get_listening_ports(show_all);
            print!("{}", render_port_table(&ports, !show_all));
            0
        }
        Ok(Command::PortDetails { port }) => {
            match get_port_details(port) {
                Some(info) => print!("{}", render_port_detail(&info)),
                None => println!("No process found listening on :{port}."),
            }
            0
        }
        Ok(Command::Kill { args }) => {
            let outcome = run_kill_command(&args);
            print!("{}", outcome.output);
            outcome.exit_code
        }
        Ok(Command::Ps { show_all }) => {
            let processes = get_all_processes(show_all);
            print!("{}", render_process_table(&processes, !show_all));
            0
        }
        Ok(Command::Logs { args }) => {
            let outcome = run_logs_command(&args);
            print!("{}", outcome.output);
            outcome.exit_code
        }
        Ok(Command::Clean) => {
            let outcome = run_clean_command();
            print!("{}", outcome.output);
            outcome.exit_code
        }
        Ok(Command::Watch) => run_watch_command(),
        Ok(Command::Help) => {
            print_help();
            0
        }
        Err(message) => {
            eprintln!("{message}");
            eprintln!("Run `ports --help` for usage.");
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    List { show_all: bool },
    PortDetails { port: u16 },
    Kill { args: Vec<String> },
    Ps { show_all: bool },
    Logs { args: Vec<String> },
    Clean,
    Watch,
    Help,
}

fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    reject_color_options(&args)?;
    let (args, show_all) = extract_show_all(args);

    if args.is_empty() {
        return Ok(Command::List { show_all });
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
            Ok(Command::Ps { show_all })
        }
        "logs" => {
            let logs_args = args.into_iter().skip(1).collect();
            Ok(Command::Logs { args: logs_args })
        }
        "clean" => {
            if args.len() > 1 {
                return Err(format!("Unknown arguments: {}", args[1..].join(" ")));
            }
            Ok(Command::Clean)
        }
        "watch" => {
            if args.len() > 1 {
                return Err(format!("Unknown arguments: {}", args[1..].join(" ")));
            }
            Ok(Command::Watch)
        }
        command => {
            if args.len() > 1 {
                return Err(format!("Unknown arguments: {}", args[1..].join(" ")));
            }
            parse_port(command).map(|port| Command::PortDetails { port })
        }
    }
}

fn reject_color_options(args: &[String]) -> Result<(), String> {
    if args
        .iter()
        .any(|arg| arg == "--color" || arg.starts_with("--color="))
    {
        return Err("Color output is always enabled; --color is not supported.".to_string());
    }

    Ok(())
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
    println!("Kiri - inspect local listening ports");
    println!();
    println!("Usage:");
    println!("  ports          Show developer listening ports");
    println!("  ports --all    Show all listening ports");
    println!("  ports <port>   Show port details");
    println!("  ports ps       Show running developer processes");
    println!("  ports logs <port|pid> [-f|--follow] [--lines N] [--err]");
    println!("  ports clean    Find orphaned/zombie dev processes and ask before killing");
    println!("  ports watch    Monitor developer port changes");
    println!("  ports kill [-f|--force] <port|pid|range> [...]");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_list_command() {
        assert_eq!(
            parse_args(Vec::<String>::new()),
            Ok(Command::List { show_all: false })
        );
    }

    #[test]
    fn parses_all_list_command() {
        assert_eq!(parse_args(["--all"]), Ok(Command::List { show_all: true }));
        assert_eq!(parse_args(["-a"]), Ok(Command::List { show_all: true }));
    }

    #[test]
    fn rejects_color_options() {
        assert_eq!(
            parse_args(["--color", "always"]),
            Err("Color output is always enabled; --color is not supported.".to_string())
        );
        assert_eq!(
            parse_args(["--all", "--color", "never"]),
            Err("Color output is always enabled; --color is not supported.".to_string())
        );
        assert_eq!(
            parse_args(["5173", "--color=never"]),
            Err("Color output is always enabled; --color is not supported.".to_string())
        );
    }

    #[test]
    fn parses_ps_command_with_all() {
        assert_eq!(parse_args(["ps"]), Ok(Command::Ps { show_all: false }));
        assert_eq!(
            parse_args(["--all", "ps"]),
            Ok(Command::Ps { show_all: true })
        );
    }

    #[test]
    fn rejects_legacy_color_auto_option() {
        assert_eq!(
            parse_args(["--color=auto"]),
            Err("Color output is always enabled; --color is not supported.".to_string())
        );
    }

    #[test]
    fn parses_deferred_commands() {
        assert_eq!(
            parse_args(["3000"]),
            Ok(Command::PortDetails { port: 3000 })
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

    #[test]
    fn parses_clean_and_watch_commands() {
        assert_eq!(parse_args(["clean"]), Ok(Command::Clean));
        assert_eq!(parse_args(["watch"]), Ok(Command::Watch));
        assert_eq!(
            parse_args(["clean", "extra"]),
            Err("Unknown arguments: extra".to_string())
        );
        assert_eq!(
            parse_args(["watch", "extra"]),
            Err("Unknown arguments: extra".to_string())
        );
    }

    #[test]
    fn port_arguments_parse_like_ports_details() {
        assert_eq!(
            parse_args(["5173"]),
            Ok(Command::PortDetails { port: 5173 })
        );
    }
}
