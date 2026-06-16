use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Wrap, Clear, Sparkline},
    Frame,
};
use crate::ui::{App, ViewMode, SortMode};

pub fn renderUi(frame: &mut Frame, app: &App) {
    match app.view_mode {
        ViewMode::Projects => {
            renderProjectsView(frame, app);
        }
        ViewMode::ConfirmKill => {
            renderListView(frame, app);
            renderConfirmDialog(frame, app);
        }
        ViewMode::Detail => {
            renderSplitView(frame, app);
        }
        ViewMode::List => {
            renderListView(frame, app);
        }
    }
}

fn renderListView(frame: &mut Frame, app: &App) {
    let chunks = if app.status_message.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.area())
    };

    renderStatsBar(frame, chunks[0], app);
    renderSearchBar(frame, chunks[1], app);
    renderTable(frame, chunks[2], app);

    if app.status_message.is_some() {
        renderStatus(frame, chunks[3], app);
        renderFooter(frame, chunks[4], app);
    } else {
        renderFooter(frame, chunks[3], app);
    }
}

fn renderSplitView(frame: &mut Frame, app: &App) {
    let vert_chunks = if app.status_message.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.area())
    };

    renderStatsBar(frame, vert_chunks[0], app);
    renderSearchBar(frame, vert_chunks[1], app);

    let horiz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(vert_chunks[2]);

    renderTable(frame, horiz_chunks[0], app);
    renderDetailPanel(frame, horiz_chunks[1], app);

    if app.status_message.is_some() {
        renderStatus(frame, vert_chunks[3], app);
        renderFooter(frame, vert_chunks[4], app);
    } else {
        renderFooter(frame, vert_chunks[3], app);
    }
}

fn renderStatsBar(frame: &mut Frame, area: Rect, app: &App) {
    let stats = app.getStats();

    let filter_info = if stats.filtered_count < stats.total_processes {
        format!("{}/{}", stats.filtered_count, stats.total_processes)
    } else {
        stats.total_processes.to_string()
    };

    let stats_line = Line::from(vec![
        Span::styled(" Proc: ", Style::default().fg(Color::DarkGray)),
        Span::styled(filter_info, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Ports: ", Style::default().fg(Color::DarkGray)),
        Span::styled(stats.unique_ports.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("CPU: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.1}%", stats.total_cpu), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Mem: ", Style::default().fg(Color::DarkGray)),
        Span::styled(formatMemory(stats.total_memory), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Conn: ", Style::default().fg(Color::DarkGray)),
        Span::styled(stats.total_connections.to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
    ]);

    frame.render_widget(Paragraph::new(stats_line), area);
}

fn renderSearchBar(frame: &mut Frame, area: Rect, app: &App) {
    let search_line = if app.search_mode {
        Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)),
        ])
    } else if !app.search_query.is_empty() {
        Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.search_query, Style::default().fg(Color::Gray)),
            Span::styled(" (press / to edit)", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::styled(" to search", Style::default().fg(Color::DarkGray)),
        ])
    };

    let paragraph = Paragraph::new(search_line);
    frame.render_widget(paragraph, area);
}

fn renderTable(frame: &mut Frame, area: Rect, app: &App) {
    let sort_indicator = |mode: SortMode| {
        if app.sort_mode == mode {
            if app.sort_reversed {
                "▲"
            } else {
                "▼"
            }
        } else {
            ""
        }
    };

    let header = Row::new(vec![
        Cell::from(format!("PORT {}", sort_indicator(SortMode::Port))).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("PID").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
        Cell::from(format!("CPU% {}", sort_indicator(SortMode::Cpu))).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from(format!("MEM {}", sort_indicator(SortMode::Memory))).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from(format!("CONN {}", sort_indicator(SortMode::Connections))).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from(format!("NAME {}", sort_indicator(SortMode::Name))).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("COMMAND").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
    ])
    .height(1);

    let table_height = area.height.saturating_sub(3) as usize;
    let visible_processes: Vec<_> = app.filtered_processes
        .iter()
        .skip(app.scroll_offset)
        .take(table_height)
        .enumerate()
        .collect();

    let rows: Vec<Row> = visible_processes.iter().map(|(offset_i, proc)| {
        let i = app.scroll_offset + offset_i;

        let port_str = format!(":{}", proc.port);
        let cpu_str = format!("{:.1}", proc.cpu_usage);
        let mem_str = formatMemory(proc.memory);
        let conn_str = if proc.connections > 0 {
            proc.connections.to_string()
        } else {
            "-".to_string()
        };

        let cmd = proc.commandDisplay();
        let cmd_truncated = if cmd.len() > 50 {
            format!("{}...", &cmd[..47])
        } else {
            cmd
        };

        let (fg, bg) = if i == app.selected {
            (Color::Black, Color::White)
        } else {
            let port_color = getPortColor(proc.port);
            (port_color, Color::Reset)
        };

        Row::new(vec![
            Cell::from(port_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Cyan })),
            Cell::from(proc.pid.to_string()),
            Cell::from(cpu_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Yellow })),
            Cell::from(mem_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Magenta })),
            Cell::from(conn_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Blue })),
            Cell::from(proc.name.clone()).style(Style::default().fg(if i == app.selected { fg } else { Color::Green })),
            Cell::from(cmd_truncated),
        ])
        .style(Style::default().fg(fg).bg(bg))
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(14),
            Constraint::Min(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
    );

    frame.render_widget(table, area);
}

fn formatMemory(bytes: u64) -> String {
    let kb = bytes / 1024;
    let mb = kb / 1024;
    if mb > 0 {
        format!("{}M", mb)
    } else {
        format!("{}K", kb)
    }
}

fn getPortColor(port: u16) -> Color {
    match port {
        80 | 443 | 8080 | 8443 => Color::Cyan,
        3306 => Color::LightMagenta,
        3000..=3999 => Color::Green,
        5432 | 5433 => Color::LightBlue,
        5000..=5999 => Color::Magenta,
        6379 => Color::Red,
        8000..=8999 => Color::Yellow,
        27017 | 27018 => Color::Blue,
        _ => Color::Gray,
    }
}

fn renderStatus(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(msg) = &app.status_message {
        let status = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(status, area);
    }
}

fn renderFooter(frame: &mut Frame, area: Rect, app: &App) {
    let help = if app.search_mode {
        Line::from(vec![
            Span::styled("Type to search", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" exit search  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" apply"),
        ])
    } else {
        match app.view_mode {
            ViewMode::List => Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" nav  "),
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(" search  "),
                Span::styled("s", Style::default().fg(Color::Magenta)),
                Span::raw(" sort  "),
                Span::styled("o", Style::default().fg(Color::Magenta)),
                Span::raw(" order  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(" detail  "),
                Span::styled("x", Style::default().fg(Color::Red)),
                Span::raw(" kill  "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(" refresh  "),
                Span::styled("q", Style::default().fg(Color::DarkGray)),
                Span::raw(" quit"),
            ]),
            ViewMode::Detail => Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back  "),
                Span::styled("x", Style::default().fg(Color::Red)),
                Span::raw(" kill  "),
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" navigate"),
            ]),
            ViewMode::ConfirmKill => Line::from(vec![
                Span::styled("y", Style::default().fg(Color::Red)),
                Span::raw(" confirm  "),
                Span::styled("n/Esc", Style::default().fg(Color::Green)),
                Span::raw(" cancel"),
            ]),
            ViewMode::Projects => Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" navigate  "),
                Span::styled("Space", Style::default().fg(Color::Green)),
                Span::raw(" start/stop  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" back  "),
                Span::styled("q", Style::default().fg(Color::DarkGray)),
                Span::raw(" quit"),
            ]),
        }
    };

    let paragraph = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn renderDetailPanel(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(proc) = app.getSelectedProcess() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
            ])
            .split(area);

        let cmd = proc.commandDisplay();

        let ascii_art = vec![
            Line::from(""),
            Line::from(Span::styled("__      _____ _   _ _ __  ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("\\ \\ /\\ / / __| | | | '_ \\ ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(" \\ V  V /\\__ \\ |_| | |_) |", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("  \\_/\\_/ |___/\\__,_| .__/ ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("                   | |    ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("                   |_|    ", Style::default().fg(Color::Cyan))),
        ];

        let ascii = Paragraph::new(ascii_art)
            .block(Block::default())
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(ascii, chunks[0]);

        let info_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&proc.name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Port: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(":{}", proc.port), Style::default().fg(Color::Cyan)),
                Span::styled("  PID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(proc.pid.to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  CPU: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.1}%", proc.cpu_usage), Style::default().fg(Color::Yellow)),
                Span::styled("  Mem: ", Style::default().fg(Color::DarkGray)),
                Span::styled(formatMemory(proc.memory), Style::default().fg(Color::Magenta)),
                Span::styled("  Conn: ", Style::default().fg(Color::DarkGray)),
                Span::styled(proc.connections.to_string(), Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&cmd, Style::default().fg(Color::Gray)),
            ]),
        ];

        let info = Paragraph::new(info_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Process Info ")
                    .border_style(Style::default().fg(Color::Cyan))
            );

        frame.render_widget(info, chunks[1]);

        if let Some(history) = app.getHistory(proc.pid) {
            let cpu_data: Vec<u64> = history.cpu.iter().map(|&v| (v * 10.0) as u64).collect();
            let cpu_sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" CPU History ({:.1}%) ", proc.cpu_usage))
                        .border_style(Style::default().fg(Color::Yellow))
                )
                .data(&cpu_data)
                .style(Style::default().fg(Color::Yellow));

            frame.render_widget(cpu_sparkline, chunks[2]);

            let mem_data: Vec<u64> = history.memory.iter().map(|&v| v / (1024 * 1024)).collect();
            let mem_sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Memory History ({}) ", formatMemory(proc.memory)))
                        .border_style(Style::default().fg(Color::Magenta))
                )
                .data(&mem_data)
                .style(Style::default().fg(Color::Magenta));

            frame.render_widget(mem_sparkline, chunks[3]);

            let conn_data: Vec<u64> = history.connections.iter().map(|&v| v as u64).collect();
            let conn_sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Connections History ({}) ", proc.connections))
                        .border_style(Style::default().fg(Color::Blue))
                )
                .data(&conn_data)
                .style(Style::default().fg(Color::Blue));

            frame.render_widget(conn_sparkline, chunks[4]);
        }
    }
}

fn renderConfirmDialog(frame: &mut Frame, app: &App) {
    if let Some(proc) = app.getSelectedProcess() {
        let area = centeredRect(45, 25, frame.area());

        frame.render_widget(Clear, area);

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Kill process?",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(&proc.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" (PID: {})", proc.pid), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(format!("Port: :{}", proc.port), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Y]", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" Yes  ", Style::default().fg(Color::Red)),
                Span::styled("[N]", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" No", Style::default().fg(Color::Green)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

fn renderProjectsView(frame: &mut Frame, app: &App) {
    let projects = app.project_manager.getProjects();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Projects Manager ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("({} saved)", projects.len()), Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(header, chunks[0]);

    let header_row = Row::new(vec![
        Cell::from("STATUS").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("NAME").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("PORT").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("PATH").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("COMMAND").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]).height(1);

    let rows: Vec<Row> = projects.iter().enumerate().map(|(i, proj)| {
        let status = if app.project_manager.isRunning(&proj.id) {
            Span::styled("● RUNNING", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("○ STOPPED", Style::default().fg(Color::DarkGray))
        };

        let style = if i == app.projects_selected {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        Row::new(vec![
            Cell::from(status),
            Cell::from(proj.name.clone()).style(Style::default().fg(if i == app.projects_selected { Color::Black } else { Color::Cyan })),
            Cell::from(format!(":{}", proj.port)).style(Style::default().fg(if i == app.projects_selected { Color::Black } else { Color::Yellow })),
            Cell::from(proj.path.clone()),
            Cell::from(proj.run_command.clone()),
        ]).style(style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Length(30),
            Constraint::Min(30),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
    );

    frame.render_widget(table, chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(" navigate  "),
        Span::styled("Space", Style::default().fg(Color::Green)),
        Span::raw(" start/stop  "),
        Span::styled("d", Style::default().fg(Color::Magenta)),
        Span::raw(" deploy  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" back  "),
        Span::styled("q", Style::default().fg(Color::DarkGray)),
        Span::raw(" quit"),
    ]))
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[2]);
}

fn centeredRect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
