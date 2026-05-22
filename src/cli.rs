use crate::kill::run_kill_command;
use crate::render::table::{render_port_detail, render_port_table};
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
    List { show_all: bool },
    PortDetails { port: u16 },
    Kill { args: Vec<String> },
    Help,
}

fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();

    if args.is_empty() {
        return Ok(Command::List { show_all: false });
    }

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(Command::Help);
    }

    if args.len() == 1 && args[0] == "--all" {
        return Ok(Command::List { show_all: true });
    }

    match args[0].as_str() {
        "kill" => {
            let kill_args = args.into_iter().skip(1).collect();
            Ok(Command::Kill { args: kill_args })
        }
        command => {
            if args.len() > 1 {
                return Err(format!("Unknown arguments: {}", args[1..].join(" ")));
            }
            parse_port(command).map(|port| Command::PortDetails { port })
        }
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
    println!("  devports kill [-f|--force] <port|pid|range> [...]");
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
