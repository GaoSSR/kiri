use crate::model::{PortInfo, ProcessListInfo};
use terminal_size::{terminal_size, Width};

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[38;2;59;130;246m";
const INDIGO: &str = "\x1b[38;2;99;102;241m";
const LIME: &str = "\x1b[38;2;101;163;13m";
const GREEN: &str = "\x1b[38;2;21;128;61m";
const RED: &str = "\x1b[38;2;239;68;68m";
const ROSE: &str = "\x1b[38;2;225;29;72m";
const PURPLE: &str = "\x1b[38;2;139;92;246m";
const ORANGE: &str = "\x1b[38;2;245;158;11m";
const BORDER_GRAY: &str = "\x1b[38;2;163;163;163m";
const BORDER_BLACK: &str = "\x1b[38;2;0;0;0m";
const DOT_GREEN: &str = "\x1b[38;2;34;197;94m";

const DEFAULT_TERMINAL_WIDTH: usize = 100;
const MIN_TERMINAL_WIDTH_FOR_KIRI_SPRITE: usize = 48;
const COLUMN_COUNT: usize = 7;
const PORT_COL: usize = 0;
const PROCESS_COL: usize = 1;
#[cfg(test)]
const PID_COL: usize = 2;
const PROJECT_COL: usize = 3;
const FRAMEWORK_COL: usize = 4;
const UPTIME_COL: usize = 5;
const STATUS_COL: usize = 6;
const BORDER_OVERHEAD: usize = 1 + COLUMN_COUNT * 3;

const HEADERS: [&str; COLUMN_COUNT] = [
    "Port",
    "Process",
    "PID",
    "Project",
    "Framework",
    "Uptime",
    "Status",
];

const PROCESS_COLUMN_COUNT: usize = 8;
const PROCESS_PID_COL: usize = 0;
const PROCESS_NAME_COL: usize = 1;
const PROCESS_CPU_COL: usize = 2;
const PROCESS_MEMORY_COL: usize = 3;
const PROCESS_PROJECT_COL: usize = 4;
const PROCESS_FRAMEWORK_COL: usize = 5;
const PROCESS_UPTIME_COL: usize = 6;
const PROCESS_DESCRIPTION_COL: usize = 7;
const PROCESS_BORDER_OVERHEAD: usize = 1 + PROCESS_COLUMN_COUNT * 3;
const PROCESS_HEADERS: [&str; PROCESS_COLUMN_COUNT] = [
    "PID",
    "Process",
    "CPU%",
    "Mem",
    "Project",
    "Framework",
    "Uptime",
    "What",
];

const COLUMN_SPECS: [ColumnSpec; COLUMN_COUNT] = [
    ColumnSpec::stable(7),
    ColumnSpec::flex(7, 5, 3),
    ColumnSpec::stable(5),
    ColumnSpec::flex(7, 4, 1),
    ColumnSpec::flex(12, 4, 2),
    ColumnSpec::stable(6),
    ColumnSpec::stable(9),
];

const PROCESS_COLUMN_SPECS: [ColumnSpec; PROCESS_COLUMN_COUNT] = [
    ColumnSpec::stable(5),
    ColumnSpec::flex(7, 5, 3),
    ColumnSpec::stable(5),
    ColumnSpec::stable(8),
    ColumnSpec::flex(7, 4, 2),
    ColumnSpec::flex(12, 4, 2),
    ColumnSpec::stable(6),
    ColumnSpec::flex(8, 5, 1),
];

#[derive(Debug, Clone, Copy)]
struct ColumnSpec {
    soft_min: usize,
    hard_min: usize,
    shrink_priority: u8,
}

impl ColumnSpec {
    const fn stable(width: usize) -> Self {
        Self {
            soft_min: width,
            hard_min: width,
            shrink_priority: u8::MAX,
        }
    }

    const fn flex(soft_min: usize, hard_min: usize, shrink_priority: u8) -> Self {
        Self {
            soft_min,
            hard_min,
            shrink_priority,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableRow {
    cells: [String; COLUMN_COUNT],
}

#[derive(Debug, Clone, PartialEq)]
struct ProcessTableRow {
    cells: [String; PROCESS_COLUMN_COUNT],
}

pub fn render_port_table(ports: &[PortInfo], filtered: bool) -> String {
    render_port_table_with_width(ports, filtered, current_terminal_width())
}

fn render_port_table_with_width(
    ports: &[PortInfo],
    filtered: bool,
    terminal_width: usize,
) -> String {
    let mut output = String::new();

    push_kiri_pet_status(
        &mut output,
        &port_pet_status(ports, filtered),
        terminal_width,
    );

    if ports.is_empty() {
        if filtered {
            output.push_str("Run ports --all to show every listener.\n");
        }
        return output;
    }

    let rows = port_rows(ports);
    let widths = calculate_column_widths(&rows, effective_port_table_width(terminal_width));
    let header = TableRow {
        cells: HEADERS.map(ToOwned::to_owned),
    };

    push_border(&mut output, &widths, BorderKind::Top);
    output.push_str(&render_table_row(&header, &widths, true));
    push_border(&mut output, &widths, BorderKind::Middle);
    for (index, row) in rows.iter().enumerate() {
        output.push_str(&render_table_row(row, &widths, false));
        if index + 1 < rows.len() {
            push_border(&mut output, &widths, BorderKind::Middle);
        }
    }
    push_border(&mut output, &widths, BorderKind::Bottom);

    output
}

fn port_pet_status(ports: &[PortInfo], filtered: bool) -> String {
    if ports.is_empty() {
        return if filtered {
            "is resting. No developer ports found.".to_string()
        } else {
            "is resting. No listening ports found.".to_string()
        };
    }

    let count = colorize(&ports.len().to_string(), RED);
    let plural = if ports.len() == 1 { "" } else { "s" };
    format!("is watching {count} port{plural}, {count} port{plural} active")
}

fn push_kiri_pet_status(output: &mut String, status: &str, terminal_width: usize) {
    if terminal_width >= MIN_TERMINAL_WIDTH_FOR_KIRI_SPRITE {
        push_kiri_terminal_sprite(output, Some(status));
    } else {
        push_kiri_status_text(output, status);
    }
    output.push_str("\n\n");
}

fn push_kiri_status_text(output: &mut String, status: &str) {
    output.push_str(&colorize_bold("Kiri", BLUE));
    output.push(' ');
    output.push_str(status);
}

fn push_kiri_terminal_sprite(output: &mut String, status: Option<&str>) {
    output.push_str("      ");
    output.push_str(&colorize(".-~~~~-.", BLUE));
    output.push('\n');

    output.push_str("   ");
    output.push_str(&colorize(".-(", BLUE));
    output.push_str("  ");
    output.push_str(&colorize("●", "\x1b[38;2;15;42;64m"));
    output.push_str("  ");
    output.push_str(&colorize("●", "\x1b[38;2;15;42;64m"));
    output.push(' ');
    output.push_str(&colorize(")-.", BLUE));
    output.push('\n');

    output.push_str("  ");
    output.push_str(&colorize("(", BLUE));
    output.push_str("  ");
    output.push_str(&colorize("•", "\x1b[38;2;255;174;186m"));
    output.push_str("   ");
    output.push_str(&colorize("⌣", "\x1b[38;2;15;42;64m"));
    output.push_str("   ");
    output.push_str(&colorize("•", "\x1b[38;2;255;174;186m"));
    output.push_str("  ");
    output.push_str(&colorize(")", BLUE));
    if let Some(status) = status {
        output.push_str("  ");
        push_kiri_status_text(output, status);
    }
    output.push('\n');

    output.push_str("   ");
    output.push_str(&colorize("'-.        .-'", BLUE));
    output.push('\n');

    output.push_str("      ");
    output.push_str(&colorize("'------'", BLUE));
    output.push('\n');
}

pub fn render_port_detail(port: &PortInfo) -> String {
    render_port_detail_with_width(port, current_terminal_width())
}

pub fn render_process_table(processes: &[ProcessListInfo], filtered: bool) -> String {
    render_process_table_with_width(processes, filtered, current_terminal_width())
}

fn render_process_table_with_width(
    processes: &[ProcessListInfo],
    filtered: bool,
    terminal_width: usize,
) -> String {
    let mut output = String::new();

    output.push_str(BOLD);
    output.push_str("Kiri");
    output.push_str(RESET);
    output.push_str(" - running processes\n\n");

    if processes.is_empty() {
        if filtered {
            output.push_str("No developer processes found.\n");
            output.push_str("Run ports ps --all to show every process.\n");
        } else {
            output.push_str("No processes found.\n");
        }
        return output;
    }

    let rows = process_rows(processes);
    let widths = calculate_process_column_widths(&rows, terminal_width);
    let header = ProcessTableRow {
        cells: PROCESS_HEADERS.map(ToOwned::to_owned),
    };

    push_process_border(&mut output, &widths, BorderKind::Top);
    output.push_str(&render_process_table_row(&header, &widths, true));
    push_process_border(&mut output, &widths, BorderKind::Middle);
    for (index, row) in rows.iter().enumerate() {
        output.push_str(&render_process_table_row(row, &widths, false));
        if index + 1 < rows.len() {
            push_process_border(&mut output, &widths, BorderKind::Middle);
        }
    }
    push_process_border(&mut output, &widths, BorderKind::Bottom);

    output.push('\n');
    output.push_str(&format!(
        "{} process{}",
        colorize(&processes.len().to_string(), GREEN),
        if processes.len() == 1 { "" } else { "es" }
    ));
    if filtered {
        output.push_str(&format!(
            " - run {} {} {} to show everything",
            colorize_bold("ports", BLUE),
            colorize("ps", PURPLE),
            colorize_bold("--all", ORANGE)
        ));
    }
    output.push('\n');

    output
}

fn render_port_detail_with_width(port: &PortInfo, terminal_width: usize) -> String {
    let mut output = String::new();

    output.push_str(&colorize("Kiri - Port ", BOLD));
    output.push_str(&colorize_bold(&port.port.to_string(), BLUE));
    output.push_str("\n\n");

    push_wrapped_field(&mut output, "Process", &port.process.name, terminal_width);
    push_wrapped_field(
        &mut output,
        "PID",
        &port.process.pid.to_string(),
        terminal_width,
    );
    push_wrapped_field(&mut output, "Status", port.status.as_str(), terminal_width);
    push_wrapped_field(
        &mut output,
        "Command",
        if port.process.command.is_empty() {
            "-"
        } else {
            &port.process.command
        },
        terminal_width,
    );
    push_wrapped_field(
        &mut output,
        "Memory",
        port.memory.as_deref().unwrap_or("-"),
        terminal_width,
    );
    push_wrapped_field(
        &mut output,
        "Uptime",
        port.uptime.as_deref().unwrap_or("-"),
        terminal_width,
    );
    push_wrapped_field(
        &mut output,
        "Directory",
        &port
            .cwd
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        terminal_width,
    );
    push_wrapped_field(
        &mut output,
        "Project",
        port.project_name.as_deref().unwrap_or("-"),
        terminal_width,
    );
    push_wrapped_field(
        &mut output,
        "Framework",
        port.framework.as_deref().unwrap_or("-"),
        terminal_width,
    );
    if port.docker_container.is_some() || port.docker_image.is_some() {
        push_wrapped_field(
            &mut output,
            "Container",
            port.docker_container.as_deref().unwrap_or("-"),
            terminal_width,
        );
        push_wrapped_field(
            &mut output,
            "Image",
            port.docker_image.as_deref().unwrap_or("-"),
            terminal_width,
        );
    }
    push_wrapped_field(
        &mut output,
        "Stop",
        &format!("To stop it, run: ports kill {}", port.port),
        terminal_width,
    );

    output
}

fn current_terminal_width() -> usize {
    terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .filter(|width| *width > 0)
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}

fn effective_port_table_width(terminal_width: usize) -> usize {
    terminal_width.max(DEFAULT_TERMINAL_WIDTH)
}

fn port_rows(ports: &[PortInfo]) -> Vec<TableRow> {
    ports
        .iter()
        .map(|port| TableRow {
            cells: [
                port.port.to_string(),
                port.process.name.clone(),
                port.process.pid.to_string(),
                port.project_name.clone().unwrap_or_else(|| "-".to_string()),
                port.framework.clone().unwrap_or_else(|| "-".to_string()),
                port.uptime.clone().unwrap_or_else(|| "-".to_string()),
                port.status.as_str().to_string(),
            ],
        })
        .collect()
}

fn process_rows(processes: &[ProcessListInfo]) -> Vec<ProcessTableRow> {
    processes
        .iter()
        .map(|process| ProcessTableRow {
            cells: [
                process.pid.to_string(),
                process.process_name.clone(),
                format!("{:.1}", process.cpu),
                process.memory.clone().unwrap_or_else(|| "-".to_string()),
                process
                    .project_name
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                process.framework.clone().unwrap_or_else(|| "-".to_string()),
                process.uptime.clone().unwrap_or_else(|| "-".to_string()),
                process.description.clone(),
            ],
        })
        .collect()
}

fn calculate_column_widths(rows: &[TableRow], terminal_width: usize) -> [usize; COLUMN_COUNT] {
    let mut widths = ideal_column_widths(rows);
    let target_content_width = terminal_width
        .saturating_sub(BORDER_OVERHEAD)
        .max(total_hard_min_width());

    if total_content_width(&widths) <= target_content_width {
        return widths;
    }

    shrink_columns(&mut widths, target_content_width, |spec| spec.soft_min);
    if total_content_width(&widths) > target_content_width {
        shrink_columns(&mut widths, target_content_width, |spec| spec.hard_min);
    }

    widths
}

fn calculate_process_column_widths(
    rows: &[ProcessTableRow],
    terminal_width: usize,
) -> [usize; PROCESS_COLUMN_COUNT] {
    let mut widths = ideal_process_column_widths(rows);
    let target_content_width = terminal_width
        .saturating_sub(PROCESS_BORDER_OVERHEAD)
        .max(total_process_hard_min_width());

    if total_process_content_width(&widths) <= target_content_width {
        return widths;
    }

    shrink_process_columns(&mut widths, target_content_width, |spec| spec.soft_min);
    if total_process_content_width(&widths) > target_content_width {
        shrink_process_columns(&mut widths, target_content_width, |spec| spec.hard_min);
    }

    widths
}

fn ideal_column_widths(rows: &[TableRow]) -> [usize; COLUMN_COUNT] {
    let mut widths = [0; COLUMN_COUNT];

    for index in 0..COLUMN_COUNT {
        widths[index] = visible_width(HEADERS[index]).max(COLUMN_SPECS[index].soft_min);
    }

    for row in rows {
        for (index, cell) in row.cells.iter().enumerate() {
            widths[index] = widths[index].max(visible_width(cell));
        }
    }

    widths
}

fn ideal_process_column_widths(rows: &[ProcessTableRow]) -> [usize; PROCESS_COLUMN_COUNT] {
    let mut widths = [0; PROCESS_COLUMN_COUNT];

    for index in 0..PROCESS_COLUMN_COUNT {
        widths[index] =
            visible_width(PROCESS_HEADERS[index]).max(PROCESS_COLUMN_SPECS[index].soft_min);
    }

    for row in rows {
        for (index, cell) in row.cells.iter().enumerate() {
            widths[index] = widths[index].max(visible_width(cell));
        }
    }

    widths
}

fn shrink_columns<F>(widths: &mut [usize; COLUMN_COUNT], target_width: usize, floor_for: F)
where
    F: Fn(ColumnSpec) -> usize,
{
    let mut priorities: Vec<u8> = COLUMN_SPECS
        .iter()
        .filter_map(|spec| (spec.shrink_priority != u8::MAX).then_some(spec.shrink_priority))
        .collect();
    priorities.sort_unstable();
    priorities.dedup();

    for priority in priorities {
        loop {
            let current = total_content_width(widths);
            if current <= target_width {
                return;
            }

            let Some(index) = (0..COLUMN_COUNT)
                .filter(|index| COLUMN_SPECS[*index].shrink_priority == priority)
                .filter(|index| widths[*index] > floor_for(COLUMN_SPECS[*index]))
                .max_by_key(|index| widths[*index] - floor_for(COLUMN_SPECS[*index]))
            else {
                break;
            };

            let floor = floor_for(COLUMN_SPECS[index]);
            let reducible = widths[index] - floor;
            let needed = current - target_width;
            widths[index] -= reducible.min(needed);
        }
    }
}

fn shrink_process_columns<F>(
    widths: &mut [usize; PROCESS_COLUMN_COUNT],
    target_width: usize,
    floor_for: F,
) where
    F: Fn(ColumnSpec) -> usize,
{
    let mut priorities: Vec<u8> = PROCESS_COLUMN_SPECS
        .iter()
        .filter_map(|spec| (spec.shrink_priority != u8::MAX).then_some(spec.shrink_priority))
        .collect();
    priorities.sort_unstable();
    priorities.dedup();

    for priority in priorities {
        loop {
            let current = total_process_content_width(widths);
            if current <= target_width {
                return;
            }

            let Some(index) = (0..PROCESS_COLUMN_COUNT)
                .filter(|index| PROCESS_COLUMN_SPECS[*index].shrink_priority == priority)
                .filter(|index| widths[*index] > floor_for(PROCESS_COLUMN_SPECS[*index]))
                .max_by_key(|index| widths[*index] - floor_for(PROCESS_COLUMN_SPECS[*index]))
            else {
                break;
            };

            let floor = floor_for(PROCESS_COLUMN_SPECS[index]);
            let reducible = widths[index] - floor;
            let needed = current - target_width;
            widths[index] -= reducible.min(needed);
        }
    }
}

fn total_hard_min_width() -> usize {
    COLUMN_SPECS.iter().map(|spec| spec.hard_min).sum()
}

fn total_content_width(widths: &[usize; COLUMN_COUNT]) -> usize {
    widths.iter().sum()
}

fn total_process_hard_min_width() -> usize {
    PROCESS_COLUMN_SPECS.iter().map(|spec| spec.hard_min).sum()
}

fn total_process_content_width(widths: &[usize; PROCESS_COLUMN_COUNT]) -> usize {
    widths.iter().sum()
}

#[cfg(test)]
fn total_table_width(widths: &[usize; COLUMN_COUNT]) -> usize {
    total_content_width(widths) + BORDER_OVERHEAD
}

#[derive(Debug, Clone, Copy)]
enum BorderKind {
    Top,
    Middle,
    Bottom,
}

fn push_border(output: &mut String, widths: &[usize; COLUMN_COUNT], kind: BorderKind) {
    let (left, fill, junction, right) = match kind {
        BorderKind::Top => ('┌', '─', '┬', '┐'),
        BorderKind::Middle => ('├', '─', '┼', '┤'),
        BorderKind::Bottom => ('└', '─', '┴', '┘'),
    };

    output.push_str(BORDER_BLACK);
    output.push(left);
    for (index, width) in widths.iter().enumerate() {
        output.push_str(&fill.to_string().repeat(width + 2));
        output.push(if index + 1 == COLUMN_COUNT {
            right
        } else {
            junction
        });
    }
    output.push_str(RESET);
    output.push('\n');
}

fn push_process_border(
    output: &mut String,
    widths: &[usize; PROCESS_COLUMN_COUNT],
    kind: BorderKind,
) {
    let (left, fill, junction, right) = match kind {
        BorderKind::Top => ('┌', '─', '┬', '┐'),
        BorderKind::Middle => ('├', '─', '┼', '┤'),
        BorderKind::Bottom => ('└', '─', '┴', '┘'),
    };

    output.push_str(BORDER_BLACK);
    output.push(left);
    for (index, width) in widths.iter().enumerate() {
        output.push_str(&fill.to_string().repeat(width + 2));
        output.push(if index + 1 == PROCESS_COLUMN_COUNT {
            right
        } else {
            junction
        });
    }
    output.push_str(RESET);
    output.push('\n');
}

fn render_table_row(row: &TableRow, widths: &[usize; COLUMN_COUNT], header: bool) -> String {
    let mut line = String::new();
    push_border_char(&mut line, '│');

    for (index, (cell, width)) in row.cells.iter().zip(widths.iter().copied()).enumerate() {
        let value = truncate_to_width(cell, width);
        let padded = pad_centered(&value, width);
        line.push(' ');
        if header {
            line.push_str(&style_header(&padded));
        } else {
            line.push_str(&style_table_cell(index, &value, &padded));
        }
        line.push(' ');
        push_border_char(&mut line, '│');
    }

    line.push('\n');
    line
}

fn render_process_table_row(
    row: &ProcessTableRow,
    widths: &[usize; PROCESS_COLUMN_COUNT],
    header: bool,
) -> String {
    let mut line = String::new();
    push_border_char(&mut line, '│');

    for (index, (cell, width)) in row.cells.iter().zip(widths.iter().copied()).enumerate() {
        let value = truncate_to_width(cell, width);
        let padded = pad_centered(&value, width);
        line.push(' ');
        if header {
            line.push_str(&style_header(&padded));
        } else {
            line.push_str(&style_process_cell(index, &value, &padded));
        }
        line.push(' ');
        push_border_char(&mut line, '│');
    }

    line.push('\n');
    line
}

fn push_border_char(output: &mut String, value: char) {
    output.push_str(BORDER_BLACK);
    output.push(value);
    output.push_str(RESET);
}

fn style_status(status: &str, padded: &str) -> String {
    let color = match status {
        "healthy" => DOT_GREEN,
        "orphaned" => ORANGE,
        "zombie" => ROSE,
        _ => INDIGO,
    };

    colorize(padded, color)
}

fn style_header(value: &str) -> String {
    colorize_bold(value, BLUE)
}

fn style_table_cell(index: usize, value: &str, padded: &str) -> String {
    match index {
        PORT_COL => colorize_bold(padded, BLUE),
        PROCESS_COL => colorize(padded, PURPLE),
        PROJECT_COL => {
            if value == "-" {
                colorize(padded, BORDER_GRAY)
            } else {
                colorize(padded, LIME)
            }
        }
        FRAMEWORK_COL => style_framework(value, padded),
        UPTIME_COL => {
            if value == "-" {
                colorize(padded, BORDER_GRAY)
            } else {
                colorize(padded, ORANGE)
            }
        }
        STATUS_COL => style_status(value, padded),
        _ => padded.to_string(),
    }
}

fn style_process_cell(index: usize, value: &str, padded: &str) -> String {
    match index {
        PROCESS_PID_COL => padded.to_string(),
        PROCESS_NAME_COL => colorize(padded, PURPLE),
        PROCESS_CPU_COL => {
            let cpu = value.parse::<f64>().unwrap_or_default();
            if cpu > 25.0 {
                colorize(padded, ROSE)
            } else if cpu > 5.0 {
                colorize(padded, ORANGE)
            } else {
                colorize(padded, LIME)
            }
        }
        PROCESS_MEMORY_COL => {
            if value == "-" {
                colorize(padded, BORDER_GRAY)
            } else {
                colorize(padded, GREEN)
            }
        }
        PROCESS_PROJECT_COL => {
            if value == "-" {
                colorize(padded, BORDER_GRAY)
            } else {
                colorize(padded, LIME)
            }
        }
        PROCESS_FRAMEWORK_COL => style_framework(value, padded),
        PROCESS_UPTIME_COL => {
            if value == "-" {
                colorize(padded, BORDER_GRAY)
            } else {
                colorize(padded, ORANGE)
            }
        }
        PROCESS_DESCRIPTION_COL => {
            if value == "-" {
                colorize(padded, BORDER_GRAY)
            } else {
                padded.to_string()
            }
        }
        _ => padded.to_string(),
    }
}

fn style_framework(framework: &str, padded: &str) -> String {
    if framework == "-" || framework.trim().is_empty() {
        return colorize(padded, BORDER_GRAY);
    }

    colorize(padded, framework_color(framework))
}

fn framework_color(_framework: &str) -> &'static str {
    RED
}

fn colorize(value: &str, color: &str) -> String {
    format!("{color}{value}{RESET}")
}

fn colorize_bold(value: &str, color: &str) -> String {
    format!("{BOLD}{color}{value}{RESET}")
}

fn pad_centered(value: &str, width: usize) -> String {
    let display_width = visible_width(value);
    let padding = width.saturating_sub(display_width);
    let left = padding / 2;
    let right = padding - left;

    format!("{}{}{}", " ".repeat(left), value, " ".repeat(right))
}

fn truncate_to_width(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if visible_width(value) <= max_width {
        return value.to_string();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let keep = max_width - 3;
    let mut output = String::with_capacity(max_width);
    for ch in value.chars().take(keep) {
        output.push(ch);
    }
    output.push_str("...");
    output
}

fn push_wrapped_field(output: &mut String, label: &str, value: &str, terminal_width: usize) {
    const LABEL_WIDTH: usize = 10;
    const GAP_WIDTH: usize = 2;

    let available = terminal_width
        .saturating_sub(LABEL_WIDTH + GAP_WIDTH)
        .max(1);
    let value = if value.is_empty() { "-" } else { value };
    let lines = limit_detail_lines(label, wrap_text(value, available), available);

    for (index, line) in lines.iter().enumerate() {
        let line = style_detail_value(label, line);
        if index == 0 {
            let label = colorize_bold(&format!("{label:<LABEL_WIDTH$}"), BLUE);
            output.push_str(&format!("{label}  {line}\n"));
        } else {
            output.push_str(&format!("{:<LABEL_WIDTH$}  {line}\n", ""));
        }
    }
}

fn limit_detail_lines(label: &str, mut lines: Vec<String>, width: usize) -> Vec<String> {
    let Some(max_lines) = detail_line_limit(label) else {
        return lines;
    };

    if lines.len() <= max_lines {
        return lines;
    }

    lines.truncate(max_lines);
    if let Some(last) = lines.last_mut() {
        *last = append_ellipsis(last, width);
    }
    lines
}

fn detail_line_limit(label: &str) -> Option<usize> {
    match label {
        "Command" => Some(6),
        "Directory" | "Image" => Some(3),
        _ => None,
    }
}

fn append_ellipsis(value: &str, width: usize) -> String {
    truncate_to_width(&format!("{value}..."), width)
}

fn style_detail_value(label: &str, value: &str) -> String {
    match label {
        "Status" => style_status(value, value),
        "Project" | "Container" => {
            if value == "-" {
                colorize(value, BORDER_GRAY)
            } else {
                colorize(value, LIME)
            }
        }
        "Framework" => style_framework(value, value),
        "Directory" => {
            if value == "-" {
                colorize(value, BORDER_GRAY)
            } else {
                colorize(value, GREEN)
            }
        }
        "Command" => style_command_line(value),
        _ => value.to_string(),
    }
}

fn style_command_line(value: &str) -> String {
    value
        .split_whitespace()
        .map(style_command_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn style_command_token(token: &str) -> String {
    if is_variable_like_token(token) {
        colorize(token, PURPLE)
    } else if token.starts_with('-') {
        colorize(token, ROSE)
    } else if is_path_or_url_token(token) {
        colorize(token, GREEN)
    } else if is_command_like_token(token) {
        colorize_bold(token, BLUE)
    } else {
        token.to_string()
    }
}

fn is_variable_like_token(token: &str) -> bool {
    token.contains('$')
        || token
            .split_once('=')
            .is_some_and(|(key, _)| key.chars().all(|ch| ch == '_' || ch.is_ascii_uppercase()))
}

fn is_path_or_url_token(token: &str) -> bool {
    token.starts_with('/')
        || token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with("http://")
        || token.starts_with("https://")
}

fn is_command_like_token(token: &str) -> bool {
    let command = token.rsplit('/').next().unwrap_or(token);
    matches!(
        command,
        "node"
            | "npm"
            | "npx"
            | "pnpm"
            | "yarn"
            | "bun"
            | "python"
            | "python3"
            | "java"
            | "docker"
            | "redis-cli"
            | "curl"
            | "cargo"
            | "go"
            | "ruby"
    )
}

fn wrap_text(value: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in value.split_whitespace() {
        let word_width = visible_width(word);
        if word_width > width {
            if !current.is_empty() {
                lines.push(current);
                current = String::new();
            }
            lines.extend(split_word(word, width));
            continue;
        }

        let next_width = if current.is_empty() {
            word_width
        } else {
            visible_width(&current) + 1 + word_width
        };

        if next_width <= width {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn split_word(value: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        current.push(ch);
        if visible_width(&current) >= width {
            chunks.push(current);
            current = String::new();
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn visible_width(value: &str) -> usize {
    strip_ansi_codes(value).chars().count()
}

fn strip_ansi_codes(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }

        output.push(ch);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PortInfo, ProcessInfo, ProcessStatus};
    use std::path::PathBuf;

    #[test]
    fn column_widths_use_ideal_widths_when_terminal_is_wide_enough() {
        let rows = vec![table_row([
            "5173", "node", "1296", "frontend", "Vite", "14h 2m", "healthy",
        ])];

        let widths = calculate_column_widths(&rows, 160);

        assert_eq!(widths[PROJECT_COL], "frontend".len());
        assert_eq!(widths[FRAMEWORK_COL], 12);
        assert!(total_table_width(&widths) <= 160);
    }

    #[test]
    fn column_widths_compress_flexible_columns_when_terminal_is_narrow() {
        let rows = vec![table_row([
            "5173",
            "very-long-node-wrapper",
            "1296",
            "project-name-that-is-far-too-long-for-a-narrow-terminal",
            "FrameworkNameThatShouldShrink",
            "14h 2m",
            "healthy",
        ])];

        let widths = calculate_column_widths(&rows, 72);

        assert!(total_table_width(&widths) <= 72);
        assert!(
            widths[PROJECT_COL] < "project-name-that-is-far-too-long-for-a-narrow-terminal".len()
        );
        assert!(widths[FRAMEWORK_COL] < "FrameworkNameThatShouldShrink".len());
    }

    #[test]
    fn column_widths_keep_short_columns_readable() {
        let rows = vec![table_row([
            ":65535",
            "node",
            "99999",
            "long-project-name-that-can-shrink",
            "Vite",
            "12d 4h",
            "healthy",
        ])];

        let widths = calculate_column_widths(&rows, 72);

        assert!(widths[PORT_COL] >= 6);
        assert!(widths[PID_COL] >= 5);
        assert!(widths[UPTIME_COL] >= 6);
        assert!(widths[STATUS_COL] >= 7);
    }

    #[test]
    fn truncate_to_width_keeps_exact_width_values() {
        assert_eq!(truncate_to_width("abcdef", 6), "abcdef");
    }

    #[test]
    fn truncate_to_width_adds_ascii_ellipsis_when_value_is_too_wide() {
        assert_eq!(truncate_to_width("abcdef", 5), "ab...");
    }

    #[test]
    fn truncate_to_width_handles_tiny_widths_without_panicking() {
        assert_eq!(truncate_to_width("abcdef", 0), "");
        assert_eq!(truncate_to_width("abcdef", 2), "..");
    }

    #[test]
    fn table_output_uses_readable_minimum_width_for_long_project_names() {
        let port = sample_port(
            "project-name-that-is-far-too-long-for-the-current-terminal-width",
            "node /Users/dev/project/node_modules/.bin/vite --host 127.0.0.1 --port 5173",
        );

        let output = render_port_table_with_width(&[port], true, 80);

        for line in output.lines() {
            assert!(
                visible_width(line) <= DEFAULT_TERMINAL_WIDTH,
                "line exceeded width: {}",
                strip_ansi_codes(line)
            );
        }
    }

    #[test]
    fn common_project_names_stay_single_line_in_narrow_port_table() {
        let port = sample_port("nori-agent-postgres-pgvector", "docker");

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 85));

        assert!(output.contains("nori-agent-postgres-pgvector"));
        assert!(!output.contains("nori-age..."));
        assert_eq!(
            output
                .lines()
                .filter(|line| line.contains("nori-agent-postgres-pgvector"))
                .count(),
            1
        );
    }

    #[test]
    fn real_default_port_view_keeps_projects_single_line_in_narrow_windows() {
        let projects = [
            (5173, "frontend"),
            (5432, "nori-agent-postgres"),
            (6379, "nori-agent-redis"),
            (8080, "backend"),
            (55433, "nori-agent-postgres-pgvector"),
        ];
        let ports: Vec<_> = projects
            .iter()
            .enumerate()
            .map(|(index, (_, project))| {
                let mut port = sample_port(project, "docker");
                port.port = projects[index].0;
                port
            })
            .collect();

        let output = strip_ansi_codes(&render_port_table_with_width(&ports, true, 85));

        for (port, project) in projects {
            assert!(output.contains(project), "{project} should render fully");
            assert_eq!(
                output
                    .lines()
                    .filter(|line| line.contains(&port.to_string()) && line.contains(project))
                    .count(),
                1,
                "{project} should render on the same single row as port {port}"
            );
        }
    }

    #[test]
    fn table_header_and_rows_have_consistent_column_separators() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);
        let table_lines: Vec<&str> = output
            .lines()
            .filter(|line| strip_ansi_codes(line).starts_with('│'))
            .collect();

        assert!(table_lines.len() >= 2);
        let separator_count = strip_ansi_codes(table_lines[0]).matches('│').count();
        for line in table_lines {
            assert_eq!(strip_ansi_codes(line).matches('│').count(), separator_count);
        }
    }

    #[test]
    fn table_primary_data_does_not_force_white_ansi_color() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(!output.contains("\x1b[37m"));
        assert!(!output.contains("\x1b[97m"));
    }

    #[test]
    fn default_rendering_keeps_ansi_for_captured_output() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b["));
        assert!(!output.contains("\x1b[37m"));
        assert!(!output.contains("\x1b[97m"));
    }

    #[test]
    fn modern_terminal_rendering_uses_title_case_headers_and_plain_ports() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 100));

        assert!(output.contains("│  Port   │"));
        assert!(output.contains("│ Process "));
        assert!(output.contains("│  Framework   │"));
        assert!(!output.contains("│ PORT "));
        assert!(output.contains("│  5173   │"));
        assert!(!output.contains(":5173"));
    }

    #[test]
    fn modern_terminal_rendering_detail_title_uses_plain_port_number() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_detail_with_width(&port, 100));

        assert!(output.contains("Kiri - Port 5173"));
        assert!(!output.contains("Port :5173"));
    }

    #[test]
    fn modern_terminal_rendering_uses_configured_syntax_palette() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        for color in [
            "\x1b[38;2;59;130;246m", // #3B82F6
            "\x1b[38;2;139;92;246m", // #8B5CF6
            "\x1b[38;2;34;197;94m",  // #22C55E
            "\x1b[38;2;245;158;11m", // #F59E0B
            "\x1b[38;2;0;0;0m",      // #000000
        ] {
            assert!(output.contains(color), "missing color {color:?}");
        }
    }

    #[test]
    fn modern_terminal_rendering_uses_distinct_process_and_project_colors() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[38;2;139;92;246m node"));
        assert!(output.contains("\x1b[38;2;101;163;13mfrontend"));
        assert!(!output.contains("\x1b[38;2;139;92;246mfrontend"));
    }

    #[test]
    fn modern_terminal_rendering_centers_headers_and_data_cells() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 100));

        assert!(output.contains("│  PID  │"));
        assert!(output.contains("│  Framework   │"));
        assert!(output.contains("│  Status   │"));
        assert!(output.contains("│ 1296  │"));
        assert!(output.contains("│  5173   │"));
        assert!(output.contains("│  node   │"));
        assert!(output.contains("│  healthy  │"));
    }

    #[test]
    fn port_table_title_uses_kiri_pet_status_for_active_ports() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 100));

        assert!(output.contains("⌣"));
        assert!(output.contains("●"));
        assert!(output.contains("Kiri is watching 1 port, 1 port active"));
        assert!(!output.contains("ports active - run"));
        assert!(!output.contains("████"));
        assert!(!output.starts_with("☁ Kiri"));
        assert!(!output.contains(".--(  o   o  )--."));
        assert!(!output.contains("Kiri - listening ports"));
    }

    #[test]
    fn port_table_empty_filtered_state_uses_resting_pet_status() {
        let output = strip_ansi_codes(&render_port_table_with_width(&[], true, 100));

        assert!(output.contains("⌣"));
        assert!(output.contains("●"));
        assert!(output.contains("Kiri is resting. No developer ports found."));
        assert!(!output.contains("████"));
        assert!(!output.starts_with("☁ Kiri"));
        assert!(!output.contains(".--(  o   o  )--."));
        assert!(output.contains("Run ports --all to show every listener."));
        assert!(!output.contains("Kiri - listening ports"));
    }

    #[test]
    fn pet_status_does_not_replace_port_detail_title() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_detail_with_width(&port, 100));

        assert!(output.starts_with("Kiri - Port 5173\n\n"));
        assert!(!output.contains("☁ Kiri is watching"));
        assert!(!output.contains(".--(  o   o  )--."));
        assert!(!output.contains('▀'));
        assert!(!output.contains('▄'));
    }

    #[test]
    fn port_table_pet_status_collapses_on_narrow_terminal() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 44));

        assert!(output.starts_with("Kiri is watching 1 port, 1 port active\n\n"));
        assert!(!output.contains("⌣"));
        assert!(!output.contains("●"));
    }

    #[test]
    fn modern_terminal_rendering_centers_wide_port_and_framework_values() {
        let mut port = sample_port("frontend", "node vite");
        port.port = 55433;
        port.framework = Some("PostgreSQL".to_string());

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 100));

        assert!(output.contains("│  55433  │"));
        assert!(output.contains("│  PostgreSQL  │"));
    }

    #[test]
    fn modern_terminal_rendering_separates_each_data_row() {
        let frontend = sample_port("frontend", "node vite");
        let mut backend = sample_port("backend", "python app.py");
        backend.port = 8000;
        backend.process.pid = 4321;
        backend.process.name = "python".to_string();
        backend.framework = Some("Python".to_string());

        let output = strip_ansi_codes(&render_port_table_with_width(
            &[frontend, backend],
            true,
            100,
        ));
        let middle_borders = output.lines().filter(|line| line.starts_with('├')).count();

        assert_eq!(middle_borders, 2);
    }

    #[test]
    fn stripped_table_output_removes_all_ansi_for_internal_assertions() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_table_with_width(&[port], true, 100));

        assert!(!output.contains("\x1b["));
        assert!(output.contains("5173"));
        assert!(!output.contains(":5173"));
        assert!(output.contains("frontend"));
    }

    #[test]
    fn stripped_detail_output_removes_all_ansi_for_internal_assertions() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_detail_with_width(&port, 100));

        assert!(!output.contains("\x1b["));
        assert!(output.contains("5173"));
        assert!(!output.contains(":5173"));
        assert!(output.contains("frontend"));
    }

    #[test]
    fn table_header_uses_colored_high_contrast_ansi() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(
            output.contains("\x1b[1m\x1b[38;2;59;130;246m Port "),
            "header should use bold bright blue"
        );
    }

    #[test]
    fn table_status_healthy_uses_bold_green_ansi() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[38;2;34;197;94m healthy"));
    }

    #[test]
    fn project_and_framework_cells_are_colored_without_breaking_width() {
        let port = sample_port(
            "project-name-that-is-far-too-long-for-the-current-terminal-width",
            "node vite",
        );

        let output = render_port_table_with_width(&[port], true, 80);
        let stripped = strip_ansi_codes(&output);

        assert!(output.contains("\x1b[38;2;101;163;13m"));
        assert!(stripped.contains("project"));
        let vite_line = output
            .lines()
            .find(|line| strip_ansi_codes(line).contains("Vite"))
            .expect("Vite row should render");
        assert!(vite_line.contains(RED));
        for line in output.lines() {
            assert!(
                visible_width(line) <= DEFAULT_TERMINAL_WIDTH,
                "line exceeded width: {}",
                strip_ansi_codes(line)
            );
        }
    }

    #[test]
    fn process_cell_uses_purple_for_command_like_identity() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[38;2;139;92;246m node"));
    }

    #[test]
    fn framework_palette_uses_uniform_red_for_all_visible_frameworks() {
        assert_eq!(framework_color("Vite"), RED);
        assert_eq!(framework_color("React"), RED);
        assert_eq!(framework_color("Redis"), RED);
        assert_eq!(framework_color("PostgreSQL"), RED);
        assert_eq!(framework_color("Docker"), RED);
        assert_eq!(framework_color("Python"), RED);
    }

    #[test]
    fn border_black_does_not_wrap_entire_table_rows() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);
        let data_line = output
            .lines()
            .find(|line| strip_ansi_codes(line).contains("5173"))
            .unwrap();

        assert!(data_line.contains(&format!("{BORDER_BLACK}│{RESET} ")));
        assert!(data_line.contains("5173"));
        assert!(!data_line.contains(":5173"));
        assert!(data_line.contains(&format!(" {BORDER_BLACK}│{RESET}")));
    }

    #[test]
    fn pet_status_highlights_counts_without_summary_hint() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[38;2;239;68;68m1\x1b[0m port"));
        assert!(output.contains(", \x1b[38;2;239;68;68m1\x1b[0m port active"));
        assert!(!output.contains("ports active - run"));
        assert!(!output.contains("show everything"));
    }

    #[test]
    fn detail_fields_use_semantic_colors_without_breaking_width() {
        let port = sample_port(
            "frontend",
            "TOKEN=$TOKEN node /Users/dev/a-very-long-project-name/node_modules/.bin/vite --host 127.0.0.1 --port 5173 --strictPort",
        );

        let output = render_port_detail_with_width(&port, 72);

        assert!(output.contains("\x1b[1m\x1b[38;2;59;130;246mProcess"));
        assert!(output.contains("\x1b[38;2;101;163;13mfrontend"));
        assert!(output.contains("\x1b[38;2;239;68;68mVite"));
        assert!(output.contains("\x1b[38;2;139;92;246mTOKEN=$TOKEN"));
        assert!(output.contains("\x1b[1m\x1b[38;2;59;130;246mnode"));
        assert!(output.contains("\x1b[38;2;21;128;61m/Users"));
        assert!(output.contains("\x1b[38;2;225;29;72m--host"));
        for line in output.lines() {
            assert!(
                visible_width(line) <= 72,
                "line exceeded width: {}",
                strip_ansi_codes(line)
            );
        }
    }

    #[test]
    fn detail_output_wraps_long_values_to_requested_width() {
        let port = sample_port(
            "frontend",
            "node /Users/dev/a-very-long-project-name/node_modules/.bin/vite --host 127.0.0.1 --port 5173 --strictPort",
        );

        let output = render_port_detail_with_width(&port, 72);

        for line in output.lines() {
            assert!(
                visible_width(line) <= 72,
                "line exceeded width: {}",
                strip_ansi_codes(line)
            );
        }
    }

    #[test]
    fn detail_output_caps_very_long_command_values() {
        let classpath = (0..40)
            .map(|index| format!("/Users/dev/.m2/repository/group/lib-{index}.jar"))
            .collect::<Vec<_>>()
            .join(":");
        let command = format!("java -cp {classpath} com.example.Main");
        let port = sample_port("backend", &command);

        let output = render_port_detail_with_width(&port, 72);
        let plain = strip_ansi_codes(&output);
        let mut command_lines = Vec::new();
        let mut inside_command = false;

        for line in plain.lines() {
            if line.starts_with("Command") {
                inside_command = true;
            } else if line.starts_with("Memory") {
                inside_command = false;
            }

            if inside_command {
                command_lines.push(line.to_string());
            }
        }

        assert_eq!(command_lines.len(), 6);
        assert!(command_lines
            .last()
            .is_some_and(|line| line.contains("...")));
        for line in output.lines() {
            assert!(
                visible_width(line) <= 72,
                "line exceeded width: {}",
                strip_ansi_codes(line)
            );
        }
    }

    #[test]
    fn detail_output_shows_explicit_ports_kill_hint_without_interaction() {
        let port = sample_port("frontend", "node vite");

        let output = strip_ansi_codes(&render_port_detail_with_width(&port, 100));

        assert!(output.contains("To stop it, run: ports kill 5173"));
        assert!(!output.contains("Kill process on"));
        assert!(!output.contains("[y/N]"));
    }

    fn table_row(cells: [&str; COLUMN_COUNT]) -> TableRow {
        TableRow {
            cells: cells.map(ToOwned::to_owned),
        }
    }

    fn sample_port(project: &str, command: &str) -> PortInfo {
        PortInfo {
            port: 5173,
            process: ProcessInfo {
                pid: 1296,
                name: "node".to_string(),
                command: command.to_string(),
                ppid: Some(100),
                stat: Some("S".to_string()),
                rss_kb: Some(12_345),
            },
            status: ProcessStatus::Healthy,
            cwd: Some(PathBuf::from("/Users/dev/project")),
            project_name: Some(project.to_string()),
            framework: Some("Vite".to_string()),
            docker_image: None,
            docker_container: None,
            memory: Some("12.1 MB".to_string()),
            uptime: Some("14h 2m".to_string()),
            start_time: None,
        }
    }
}
