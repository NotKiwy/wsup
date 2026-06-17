use crate::core::{findProcessByPort, PortQueryError, ProcessInfo};
use crate::utils::{formatMemory, LOGO_LINES};
use crossterm::style::{Color, Stylize};

pub fn printPortInfo(port: u16) {
    match findProcessByPort(port) {
        Ok(proc) => printProcessSummary(&proc),
        Err(PortQueryError::NotFound) => {
            println!("Port {} is not in use.", port);
        }
        Err(PortQueryError::PermissionDenied(message)) => {
            eprintln!("{} {}", "Error:".with(Color::Red), message);
            std::process::exit(1);
        }
    }
}

fn terminalWidth() -> usize {
    crossterm::terminal::size()
        .map(|(cols, _)| cols as usize)
        .unwrap_or(80)
        .max(40)
}

fn printCenteredColored(text: &str, width: usize, color: Color) {
    let pad = width.saturating_sub(text.chars().count()) / 2;
    println!("{}{}", " ".repeat(pad), text.with(color));
}

fn printBoxLine(margin: &str, inner_width: usize, plain: &str, styled: &str) {
    let pad = inner_width.saturating_sub(plain.chars().count());
    println!(
        "{}{} {}{} {}",
        margin,
        "│".with(Color::Cyan),
        styled,
        " ".repeat(pad),
        "│".with(Color::Cyan)
    );
}

fn printBoxBlank(margin: &str, inner_width: usize) {
    println!(
        "{}{} {} {}",
        margin,
        "│".with(Color::Cyan),
        " ".repeat(inner_width),
        "│".with(Color::Cyan)
    );
}

fn printProcessSummary(proc: &ProcessInfo) {
    let term_width = terminalWidth();

    println!();
    for line in LOGO_LINES {
        printCenteredColored(line, term_width, Color::Cyan);
    }
    println!();

    let name_plain = proc.name.clone();
    let name_styled = format!("{}", proc.name.clone().with(Color::Green));

    let port_plain = format!(
        "Port: :{}   PID: {}   Protocol: {}",
        proc.port, proc.pid, proc.protocol
    );
    let port_styled = format!(
        "{}{}{}{}{}{}",
        "Port: ".with(Color::DarkGrey),
        format!(":{}", proc.port).with(Color::Cyan),
        "   PID: ".with(Color::DarkGrey),
        proc.pid.to_string().with(Color::White),
        "   Protocol: ".with(Color::DarkGrey),
        proc.protocol.clone().with(Color::White),
    );

    let stats_plain = format!(
        "CPU: {:.1}%   Mem: {}   Conn: {}",
        proc.cpu_usage,
        formatMemory(proc.memory),
        proc.connections
    );
    let stats_styled = format!(
        "{}{}{}{}{}{}",
        "CPU: ".with(Color::DarkGrey),
        format!("{:.1}%", proc.cpu_usage).with(Color::Yellow),
        "   Mem: ".with(Color::DarkGrey),
        formatMemory(proc.memory).with(Color::Magenta),
        "   Conn: ".with(Color::DarkGrey),
        proc.connections.to_string().with(Color::Blue),
    );

    let exe_plain = proc.exe_path.as_ref().map(|p| format!("Executable: {}", p));
    let exe_styled = proc.exe_path.as_ref().map(|p| {
        format!(
            "{}{}",
            "Executable: ".with(Color::DarkGrey),
            p.clone().with(Color::Grey),
        )
    });

    let title = "Process Info";
    let mut max_content = [&name_plain, &port_plain, &stats_plain]
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    if let Some(ref exe) = exe_plain {
        max_content = max_content.max(exe.chars().count());
    }

    let title_min_inner = title.chars().count() + 2;
    let inner_width = max_content
        .max(title_min_inner)
        .min(term_width.saturating_sub(6))
        .max(title_min_inner);

    let box_width = inner_width + 4;
    let left_margin = term_width.saturating_sub(box_width) / 2;
    let margin = " ".repeat(left_margin);

    let title_segment = format!("─ {} ", title);
    let title_len = title_segment.chars().count();
    let top_dashes = (box_width.saturating_sub(2)).saturating_sub(title_len);
    println!(
        "{}{}",
        margin,
        format!("┌{}{}┐", title_segment, "─".repeat(top_dashes)).with(Color::Cyan)
    );

    printBoxBlank(&margin, inner_width);
    printBoxLine(&margin, inner_width, &name_plain, &name_styled);
    printBoxBlank(&margin, inner_width);
    printBoxLine(&margin, inner_width, &port_plain, &port_styled);
    printBoxBlank(&margin, inner_width);
    printBoxLine(&margin, inner_width, &stats_plain, &stats_styled);
    if let (Some(plain), Some(styled)) = (exe_plain, exe_styled) {
        printBoxBlank(&margin, inner_width);
        printBoxLine(&margin, inner_width, &plain, &styled);
    }
    printBoxBlank(&margin, inner_width);

    println!(
        "{}{}",
        margin,
        format!("└{}┘", "─".repeat(box_width.saturating_sub(2))).with(Color::Cyan)
    );
    println!();
}