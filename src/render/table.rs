use crate::model::{PortInfo, ProcessListInfo};
use std::io::IsTerminal;
use terminal_size::{terminal_size, Width};

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const BOLD_CYAN: &str = "\x1b[1;36m";
const BOLD_BLUE: &str = "\x1b[1;34m";
const GREEN: &str = "\x1b[1;32m";
const TOKEN_GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[1;33m";
const RED: &str = "\x1b[1;31m";
const TOKEN_RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[1;34m";
const PURPLE: &str = "\x1b[1;35m";

const DEFAULT_TERMINAL_WIDTH: usize = 100;
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
    "PORT",
    "PROCESS",
    "PID",
    "PROJECT",
    "FRAMEWORK",
    "UPTIME",
    "STATUS",
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
    "PROCESS",
    "CPU%",
    "MEM",
    "PROJECT",
    "FRAMEWORK",
    "UPTIME",
    "WHAT",
];

const COLUMN_SPECS: [ColumnSpec; COLUMN_COUNT] = [
    ColumnSpec::stable(6, Align::Left),
    ColumnSpec::flex(7, 5, 3, Align::Left),
    ColumnSpec::stable(5, Align::Right),
    ColumnSpec::flex(7, 4, 1, Align::Left),
    ColumnSpec::flex(9, 4, 2, Align::Left),
    ColumnSpec::stable(6, Align::Left),
    ColumnSpec::stable(7, Align::Left),
];

const PROCESS_COLUMN_SPECS: [ColumnSpec; PROCESS_COLUMN_COUNT] = [
    ColumnSpec::stable(5, Align::Right),
    ColumnSpec::flex(7, 5, 3, Align::Left),
    ColumnSpec::stable(5, Align::Right),
    ColumnSpec::stable(8, Align::Right),
    ColumnSpec::flex(7, 4, 2, Align::Left),
    ColumnSpec::flex(9, 4, 2, Align::Left),
    ColumnSpec::stable(6, Align::Left),
    ColumnSpec::flex(8, 5, 1, Align::Left),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
struct ColumnSpec {
    soft_min: usize,
    hard_min: usize,
    shrink_priority: u8,
    align: Align,
}

impl ColumnSpec {
    const fn stable(width: usize, align: Align) -> Self {
        Self {
            soft_min: width,
            hard_min: width,
            shrink_priority: u8::MAX,
            align,
        }
    }

    const fn flex(soft_min: usize, hard_min: usize, shrink_priority: u8, align: Align) -> Self {
        Self {
            soft_min,
            hard_min,
            shrink_priority,
            align,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    fn should_color(self) -> bool {
        match self {
            Self::Auto => std::io::stdout().is_terminal(),
            Self::Always => true,
            Self::Never => false,
        }
    }
}

pub fn render_port_table(ports: &[PortInfo], filtered: bool) -> String {
    render_port_table_with_color_mode(ports, filtered, ColorMode::Auto)
}

pub fn render_port_table_with_color_mode(
    ports: &[PortInfo],
    filtered: bool,
    color_mode: ColorMode,
) -> String {
    apply_color_mode(
        render_port_table_with_width(ports, filtered, current_terminal_width()),
        color_mode,
    )
}

fn render_port_table_with_width(
    ports: &[PortInfo],
    filtered: bool,
    terminal_width: usize,
) -> String {
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

    let rows = port_rows(ports);
    let widths = calculate_column_widths(&rows, terminal_width);
    let header = TableRow {
        cells: HEADERS.map(ToOwned::to_owned),
    };

    push_border(&mut output, &widths, BorderKind::Top);
    output.push_str(&render_table_row(&header, &widths, true));
    push_border(&mut output, &widths, BorderKind::Middle);
    for row in &rows {
        output.push_str(&render_table_row(row, &widths, false));
    }
    push_border(&mut output, &widths, BorderKind::Bottom);

    output.push('\n');
    output.push_str(&format!(
        "{} port{} active",
        colorize(&ports.len().to_string(), GREEN),
        if ports.len() == 1 { "" } else { "s" }
    ));
    if filtered {
        output.push_str(&format!(
            " - run `{}` `{}` to show everything",
            colorize("devports", BOLD_CYAN),
            colorize("--all", YELLOW)
        ));
    }
    output.push('\n');

    output
}

pub fn render_port_detail(port: &PortInfo) -> String {
    render_port_detail_with_color_mode(port, ColorMode::Auto)
}

pub fn render_port_detail_with_color_mode(port: &PortInfo, color_mode: ColorMode) -> String {
    apply_color_mode(
        render_port_detail_with_width(port, current_terminal_width()),
        color_mode,
    )
}

pub fn render_process_table_with_color_mode(
    processes: &[ProcessListInfo],
    filtered: bool,
    color_mode: ColorMode,
) -> String {
    apply_color_mode(
        render_process_table_with_width(processes, filtered, current_terminal_width()),
        color_mode,
    )
}

fn render_process_table_with_width(
    processes: &[ProcessListInfo],
    filtered: bool,
    terminal_width: usize,
) -> String {
    let mut output = String::new();

    output.push_str(BOLD);
    output.push_str("DevPorts");
    output.push_str(RESET);
    output.push_str(" - running processes\n\n");

    if processes.is_empty() {
        if filtered {
            output.push_str("No developer processes found.\n");
            output.push_str("Run `devports ps --all` to show every process.\n");
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
    for row in &rows {
        output.push_str(&render_process_table_row(row, &widths, false));
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
            " - run `{}` `{}` `{}` to show everything",
            colorize("devports", BOLD_CYAN),
            colorize("ps", PURPLE),
            colorize("--all", YELLOW)
        ));
    }
    output.push('\n');

    output
}

fn render_port_detail_with_width(port: &PortInfo, terminal_width: usize) -> String {
    let mut output = String::new();

    output.push_str(&colorize("DevPorts - Port ", BOLD));
    output.push_str(&colorize(&format!(":{}", port.port), BOLD_CYAN));
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

    output
}

fn apply_color_mode(output: String, color_mode: ColorMode) -> String {
    if color_mode.should_color() {
        output
    } else {
        strip_ansi_codes(&output)
    }
}

fn current_terminal_width() -> usize {
    terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .filter(|width| *width > 0)
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}

fn port_rows(ports: &[PortInfo]) -> Vec<TableRow> {
    ports
        .iter()
        .map(|port| TableRow {
            cells: [
                format!(":{}", port.port),
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

    output.push_str(DIM);
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

    output.push_str(DIM);
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

    for index in 0..COLUMN_COUNT {
        let value = truncate_to_width(&row.cells[index], widths[index]);
        let padded = pad_aligned(&value, widths[index], COLUMN_SPECS[index].align);
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

    for index in 0..PROCESS_COLUMN_COUNT {
        let value = truncate_to_width(&row.cells[index], widths[index]);
        let padded = pad_aligned(&value, widths[index], PROCESS_COLUMN_SPECS[index].align);
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
    output.push_str(DIM);
    output.push(value);
    output.push_str(RESET);
}

fn style_status(status: &str, padded: &str) -> String {
    let color = match status {
        "healthy" => GREEN,
        "orphaned" => YELLOW,
        "zombie" => RED,
        _ => BLUE,
    };

    format!("{color}{padded}{RESET}")
}

fn style_header(value: &str) -> String {
    colorize(value, BOLD_CYAN)
}

fn style_table_cell(index: usize, value: &str, padded: &str) -> String {
    match index {
        PORT_COL => colorize(padded, BOLD_CYAN),
        PROCESS_COL => colorize(padded, PURPLE),
        PROJECT_COL => {
            if value == "-" {
                colorize(padded, DIM)
            } else {
                colorize(padded, BOLD_BLUE)
            }
        }
        FRAMEWORK_COL => style_framework(value, padded),
        UPTIME_COL => {
            if value == "-" {
                colorize(padded, DIM)
            } else {
                colorize(padded, YELLOW)
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
                colorize(padded, RED)
            } else if cpu > 5.0 {
                colorize(padded, YELLOW)
            } else {
                colorize(padded, GREEN)
            }
        }
        PROCESS_MEMORY_COL => {
            if value == "-" {
                colorize(padded, DIM)
            } else {
                colorize(padded, TOKEN_GREEN)
            }
        }
        PROCESS_PROJECT_COL => {
            if value == "-" {
                colorize(padded, DIM)
            } else {
                colorize(padded, BOLD_BLUE)
            }
        }
        PROCESS_FRAMEWORK_COL => style_framework(value, padded),
        PROCESS_UPTIME_COL => {
            if value == "-" {
                colorize(padded, DIM)
            } else {
                colorize(padded, YELLOW)
            }
        }
        PROCESS_DESCRIPTION_COL => {
            if value == "-" {
                colorize(padded, DIM)
            } else {
                padded.to_string()
            }
        }
        _ => padded.to_string(),
    }
}

fn style_framework(framework: &str, padded: &str) -> String {
    if framework == "-" || framework.trim().is_empty() {
        return colorize(padded, DIM);
    }

    colorize(padded, framework_color(framework))
}

fn framework_color(framework: &str) -> &'static str {
    let lower = framework.to_ascii_lowercase();

    if lower.contains("vite") || lower.contains("python") {
        YELLOW
    } else if lower.contains("postgres") || lower.contains("docker") || lower.contains("nginx") {
        BLUE
    } else if lower.contains("redis") || lower.contains("java") {
        RED
    } else if lower.contains("react")
        || lower.contains("next")
        || lower.contains("vue")
        || lower.contains("svelte")
        || lower.contains("remix")
        || lower.contains("astro")
    {
        PURPLE
    } else {
        BOLD_CYAN
    }
}

fn colorize(value: &str, color: &str) -> String {
    format!("{color}{value}{RESET}")
}

fn pad_aligned(value: &str, width: usize, align: Align) -> String {
    let display_width = visible_width(value);
    let padding = width.saturating_sub(display_width);

    match align {
        Align::Left => format!("{value}{}", " ".repeat(padding)),
        Align::Right => format!("{}{value}", " ".repeat(padding)),
    }
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
    let lines = wrap_text(value, available);

    for (index, line) in lines.iter().enumerate() {
        let line = style_detail_value(label, line);
        if index == 0 {
            let label = colorize(&format!("{label:<LABEL_WIDTH$}"), BOLD_CYAN);
            output.push_str(&format!("{label}  {line}\n"));
        } else {
            output.push_str(&format!("{:<LABEL_WIDTH$}  {line}\n", ""));
        }
    }
}

fn style_detail_value(label: &str, value: &str) -> String {
    match label {
        "Status" => style_status(value, value),
        "Project" | "Container" => {
            if value == "-" {
                colorize(value, DIM)
            } else {
                colorize(value, BOLD_BLUE)
            }
        }
        "Framework" => style_framework(value, value),
        "Directory" => {
            if value == "-" {
                colorize(value, DIM)
            } else {
                colorize(value, BLUE)
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
        colorize(token, TOKEN_RED)
    } else if is_path_or_url_token(token) {
        colorize(token, TOKEN_GREEN)
    } else if is_command_like_token(token) {
        colorize(token, BOLD_CYAN)
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
            ":5173", "node", "1296", "frontend", "Vite", "14h 2m", "healthy",
        ])];

        let widths = calculate_column_widths(&rows, 160);

        assert_eq!(widths[PROJECT_COL], "frontend".len());
        assert_eq!(widths[FRAMEWORK_COL], "FRAMEWORK".len());
        assert!(total_table_width(&widths) <= 160);
    }

    #[test]
    fn column_widths_compress_flexible_columns_when_terminal_is_narrow() {
        let rows = vec![table_row([
            ":5173",
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
    fn table_output_does_not_exceed_requested_width_for_long_project_names() {
        let port = sample_port(
            "project-name-that-is-far-too-long-for-the-current-terminal-width",
            "node /Users/dev/project/node_modules/.bin/vite --host 127.0.0.1 --port 5173",
        );

        let output = render_port_table_with_width(&[port], true, 80);

        for line in output.lines() {
            assert!(
                visible_width(line) <= 80,
                "line exceeded width: {}",
                strip_ansi_codes(line)
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
    fn color_mode_always_keeps_ansi_for_captured_output() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_color_mode(&[port], true, ColorMode::Always);

        assert!(output.contains("\x1b["));
        assert!(!output.contains("\x1b[37m"));
        assert!(!output.contains("\x1b[97m"));
    }

    #[test]
    fn color_mode_never_removes_all_ansi() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_color_mode(&[port], true, ColorMode::Never);

        assert!(!output.contains("\x1b["));
        assert!(output.contains(":5173"));
        assert!(output.contains("frontend"));
    }

    #[test]
    fn detail_color_mode_never_removes_all_ansi() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_detail_with_color_mode(&port, ColorMode::Never);

        assert!(!output.contains("\x1b["));
        assert!(output.contains(":5173"));
        assert!(output.contains("frontend"));
    }

    #[test]
    fn table_header_uses_colored_high_contrast_ansi() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(
            output.contains("\x1b[1;36m") || output.contains("\x1b[1;34m"),
            "header should use bold cyan or bold blue"
        );
    }

    #[test]
    fn table_status_healthy_uses_bold_green_ansi() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[1;32mhealthy"));
    }

    #[test]
    fn project_and_framework_cells_are_colored_without_breaking_width() {
        let port = sample_port(
            "project-name-that-is-far-too-long-for-the-current-terminal-width",
            "node vite",
        );

        let output = render_port_table_with_width(&[port], true, 80);

        assert!(output.contains("\x1b[1;34mproject"));
        assert!(output.contains("\x1b[1;33mVite"));
        for line in output.lines() {
            assert!(
                visible_width(line) <= 80,
                "line exceeded width: {}",
                strip_ansi_codes(line)
            );
        }
    }

    #[test]
    fn process_cell_uses_purple_for_command_like_identity() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[1;35mnode"));
    }

    #[test]
    fn border_dim_does_not_wrap_entire_table_rows() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);
        let data_line = output
            .lines()
            .find(|line| strip_ansi_codes(line).contains(":5173"))
            .unwrap();

        assert!(data_line.contains(&format!("{DIM}│{RESET} ")));
        assert!(data_line.contains(":5173"));
        assert!(data_line.contains(&format!(" {DIM}│{RESET}")));
    }

    #[test]
    fn summary_highlights_count_command_and_all_flag() {
        let port = sample_port("frontend", "node vite");

        let output = render_port_table_with_width(&[port], true, 100);

        assert!(output.contains("\x1b[1;32m1\x1b[0m port active"));
        assert!(output.contains("\x1b[1;36mdevports\x1b[0m"));
        assert!(output.contains("\x1b[1;33m--all\x1b[0m"));
    }

    #[test]
    fn detail_fields_use_semantic_colors_without_breaking_width() {
        let port = sample_port(
            "frontend",
            "TOKEN=$TOKEN node /Users/dev/a-very-long-project-name/node_modules/.bin/vite --host 127.0.0.1 --port 5173 --strictPort",
        );

        let output = render_port_detail_with_width(&port, 72);

        assert!(output.contains("\x1b[1;36mProcess"));
        assert!(output.contains("\x1b[1;34mfrontend"));
        assert!(output.contains("\x1b[1;33mVite"));
        assert!(output.contains("\x1b[1;35mTOKEN=$TOKEN"));
        assert!(output.contains("\x1b[1;36mnode"));
        assert!(output.contains("\x1b[32m/Users"));
        assert!(output.contains("\x1b[31m--host"));
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
