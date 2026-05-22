use crate::model::PortInfo;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub fn render_port_table(ports: &[PortInfo], filtered: bool) -> String {
    let mut output = String::new();

    output.push_str(BOLD);
    output.push_str("DevPorts");
    output.push_str(RESET);
    output.push_str(" - listening ports\n\n");

    if ports.is_empty() {
        if filtered {
            output.push_str("No developer listening ports found.\n");
            output.push_str("Run `devports --all` to show every listener.\n");
        } else {
            output.push_str("No listening ports found.\n");
        }
        return output;
    }

    let rows: Vec<[String; 6]> = ports
        .iter()
        .map(|port| {
            [
                format!(":{}", port.port),
                truncate(&port.process.name, 32),
                port.process.pid.to_string(),
                port.project_name
                    .as_deref()
                    .map(|name| truncate(name, 24))
                    .unwrap_or_else(|| "-".to_string()),
                port.uptime.clone().unwrap_or_else(|| "-".to_string()),
                port.status.as_str().to_string(),
            ]
        })
        .collect();

    let widths = column_widths(
        &["PORT", "PROCESS", "PID", "PROJECT", "UPTIME", "STATUS"],
        &rows,
    );
    push_row(
        &mut output,
        &widths,
        ["PORT", "PROCESS", "PID", "PROJECT", "UPTIME", "STATUS"],
        true,
    );
    push_separator(&mut output, &widths);
    for row in &rows {
        push_row(
            &mut output,
            &widths,
            [&row[0], &row[1], &row[2], &row[3], &row[4], &row[5]],
            false,
        );
    }

    output.push('\n');
    output.push_str(&format!(
        "{} port{} active",
        ports.len(),
        if ports.len() == 1 { "" } else { "s" }
    ));
    if filtered {
        output.push_str(" - run `devports --all` to show everything");
    }
    output.push('\n');

    output
}

pub fn render_port_detail(port: &PortInfo) -> String {
    let mut output = String::new();

    output.push_str(BOLD);
    output.push_str(&format!("DevPorts - Port :{}", port.port));
    output.push_str(RESET);
    output.push_str("\n\n");

    push_field(&mut output, "Process", &port.process.name);
    push_field(&mut output, "PID", &port.process.pid.to_string());
    push_field(&mut output, "Status", port.status.as_str());
    push_field(
        &mut output,
        "Command",
        if port.process.command.is_empty() {
            "-"
        } else {
            &port.process.command
        },
    );
    push_field(&mut output, "Memory", port.memory.as_deref().unwrap_or("-"));
    push_field(&mut output, "Uptime", port.uptime.as_deref().unwrap_or("-"));
    push_field(
        &mut output,
        "Directory",
        &port
            .cwd
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
    );
    push_field(
        &mut output,
        "Project",
        port.project_name.as_deref().unwrap_or("-"),
    );

    output
}

fn push_field(output: &mut String, label: &str, value: &str) {
    output.push_str(&format!("{label:<10}  {value}\n"));
}

fn column_widths(headers: &[&str; 6], rows: &[[String; 6]]) -> [usize; 6] {
    let mut widths = [
        headers[0].len(),
        headers[1].len(),
        headers[2].len(),
        headers[3].len(),
        headers[4].len(),
        headers[5].len(),
    ];

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.len());
        }
    }

    widths
}

fn push_row<T>(output: &mut String, widths: &[usize; 6], cells: [T; 6], header: bool)
where
    T: AsRef<str>,
{
    if header {
        output.push_str(BOLD);
    }
    output.push_str(&format!(
        "{:<w0$}  {:<w1$}  {:>w2$}  {:<w3$}  {:<w4$}  {:<w5$}\n",
        cells[0].as_ref(),
        cells[1].as_ref(),
        cells[2].as_ref(),
        cells[3].as_ref(),
        cells[4].as_ref(),
        cells[5].as_ref(),
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2],
        w3 = widths[3],
        w4 = widths[4],
        w5 = widths[5],
    ));
    if header {
        output.push_str(RESET);
    }
}

fn push_separator(output: &mut String, widths: &[usize; 6]) {
    output.push_str(&format!(
        "{:-<w0$}  {:-<w1$}  {:-<w2$}  {:-<w3$}  {:-<w4$}  {:-<w5$}\n",
        "",
        "",
        "",
        "",
        "",
        "",
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2],
        w3 = widths[3],
        w4 = widths[4],
        w5 = widths[5],
    ));
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}
