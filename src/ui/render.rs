use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Clear, Sparkline},
    Frame,
};
use crate::ui::{App, ViewMode, SortMode};
use crate::utils::{formatMemory, LOGO_LINES};

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
            Constraint::Percentage(55),
            Constraint::Percentage(45),
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
            if app.sort_reversed { "▲" } else { "▼" }
        } else {
            ""
        }
    };

    let total_width = area.width as usize;
    let fixed_cols: usize = 9 + 7 + 6 + 7 + 6;
    let borders: usize = 2;
    let col_gaps: usize = 6; 
    let overhead = fixed_cols + borders + col_gaps;

    let remaining = total_width.saturating_sub(overhead);

    let name_width = remaining.min(20);
    let command_width = remaining.saturating_sub(name_width);

    let header = Row::new(vec![
        Cell::from(format!("PORT {}", sort_indicator(SortMode::Port)))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("PID")
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
        Cell::from(format!("CPU% {}", sort_indicator(SortMode::Cpu)))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from(format!("MEM {}", sort_indicator(SortMode::Memory)))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from(format!("CONN {}", sort_indicator(SortMode::Connections)))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from(format!("NAME {}", sort_indicator(SortMode::Name)))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("COMMAND")
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
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
        let pid_str = proc.pid.to_string();
        let cpu_str = format!("{:.1}", proc.cpu_usage);
        let mem_str = formatMemory(proc.memory);
        let conn_str = if proc.connections > 0 {
            proc.connections.to_string()
        } else {
            "-".to_string()
        };

        let cmd = proc.commandDisplay();
        let cmd_truncated = truncateWithEllipsis(&cmd, command_width);
        let name_truncated = truncateWithEllipsis(&proc.name, name_width);

        let (fg, bg) = if i == app.selected {
            (Color::Black, Color::White)
        } else {
            let port_color = getPortColor(proc.port);
            (port_color, Color::Reset)
        };

        Row::new(vec![
            Cell::from(port_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Cyan })),
            Cell::from(pid_str).style(Style::default().fg(if i == app.selected { fg } else { Color::DarkGray })),
            Cell::from(cpu_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Yellow })),
            Cell::from(mem_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Magenta })),
            Cell::from(conn_str).style(Style::default().fg(if i == app.selected { fg } else { Color::Blue })),
            Cell::from(name_truncated).style(Style::default().fg(if i == app.selected { fg } else { Color::Green })),
            Cell::from(cmd_truncated).style(Style::default().fg(if i == app.selected { fg } else { Color::Gray })),
        ])
        .style(Style::default().fg(fg).bg(bg))
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(9),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(name_width as u16),
            Constraint::Min(command_width as u16),
        ],
    )
    .column_spacing(1)
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
    );

    frame.render_widget(table, area);
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
    let mut help = if app.search_mode {
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
                Span::raw(" navigate  "),
                Span::styled("q", Style::default().fg(Color::DarkGray)),
                Span::raw(" quit"),
            ]),
            ViewMode::ConfirmKill => {
                if app.isSelectedDocker() && !app.force_kill {
                    Line::from(vec![
                        Span::styled("f", Style::default().fg(Color::Yellow)),
                        Span::raw(" enable force  "),
                        Span::styled("n/Esc", Style::default().fg(Color::Green)),
                        Span::raw(" cancel"),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("y", Style::default().fg(Color::Red)),
                        Span::raw(" confirm  "),
                        Span::styled("n/Esc", Style::default().fg(Color::Green)),
                        Span::raw(" cancel"),
                    ])
                }
            }
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

    if let Some(ref version) = app.update_available {
        help.spans.push(Span::raw("  "));
        help.spans.push(Span::styled(
            format!("Update available: v{}", version),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ));
    }

    let paragraph = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn truncateWithEllipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let char_count = text.chars().count();
    if char_count <= max_width {
        return text.to_string();
    }
    if max_width == 1 {
        return "…".to_string();
    }
    let keep = max_width - 1;
    let truncated: String = text.chars().take(keep).collect();
    format!("{}…", truncated)
}

fn renderDetailPanel(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(proc) = app.getSelectedProcess() {
        let total_h = area.height as usize;
        let sparkline_h: usize = 5;
        let info_h: usize = 13;
        let logo_h: usize = 8;
        let sparklines_total = sparkline_h * 3;

        let (actual_logo_h, actual_info_h) = if total_h >= logo_h + info_h + sparklines_total {
            (logo_h, info_h)
        } else if total_h >= info_h + sparklines_total {
            (0, info_h)
        } else {
            let info_available = total_h.saturating_sub(sparklines_total).max(4);
            (0, info_available)
        };

        let mut constraints: Vec<Constraint> = Vec::new();
        if actual_logo_h > 0 {
            constraints.push(Constraint::Length(actual_logo_h as u16));
        }
        if actual_info_h > 0 {
            constraints.push(Constraint::Length(actual_info_h as u16));
        }

        let used = actual_logo_h + actual_info_h;
        let leftover = total_h.saturating_sub(used);
        if leftover >= sparklines_total {
            constraints.push(Constraint::Length(sparkline_h as u16));
            constraints.push(Constraint::Length(sparkline_h as u16));
            constraints.push(Constraint::Min(sparkline_h as u16));
        } else if leftover > 0 {
            constraints.push(Constraint::Ratio(1, 3));
            constraints.push(Constraint::Ratio(1, 3));
            constraints.push(Constraint::Min(1));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints.clone())
            .split(area);

        let mut chunk_idx: usize = 0;

        if actual_logo_h > 0 {
            let ascii_art: Vec<Line> = std::iter::once(Line::from(""))
                .chain(LOGO_LINES.iter().map(|line| {
                    Line::from(Span::styled(*line, Style::default().fg(Color::Cyan)))
                }))
                .collect();

            let ascii = Paragraph::new(ascii_art)
                .block(Block::default())
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(ascii, chunks[chunk_idx]);
            chunk_idx += 1;
        }

        if actual_info_h > 0 {
            let cmd = proc.commandDisplay();
            let text_width = (chunks[chunk_idx].width as usize).saturating_sub(4);

            let mut info_text = vec![
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
                    Span::styled(" Protocol: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(proc.protocol.clone(), Style::default().fg(Color::White)),
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
            ];

            let label_exe = "Executable: ";
            let label_cmd = "Command: ";

            if let Some(exe) = &proc.exe_path {
                let max = text_width.saturating_sub(label_exe.len());
                let exe_display = truncateWithEllipsis(exe, max);
                info_text.push(Line::from(vec![
                    Span::styled(format!("  {}", label_exe), Style::default().fg(Color::DarkGray)),
                    Span::styled(exe_display, Style::default().fg(Color::Gray)),
                ]));
            }

            let max_cmd = text_width.saturating_sub(label_cmd.len());
            let cmd_display = truncateWithEllipsis(&cmd, max_cmd);
            info_text.push(Line::from(vec![
                Span::styled(format!("  {}", label_cmd), Style::default().fg(Color::DarkGray)),
                Span::styled(cmd_display, Style::default().fg(Color::Gray)),
            ]));

            let info = Paragraph::new(info_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Process Info ")
                        .border_style(Style::default().fg(Color::Cyan))
                );

            frame.render_widget(info, chunks[chunk_idx]);
            chunk_idx += 1;
        }

        if chunk_idx < chunks.len() {
            if let Some(history) = app.getHistory(proc.pid) {
                if chunk_idx < chunks.len() {
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
                    frame.render_widget(cpu_sparkline, chunks[chunk_idx]);
                    chunk_idx += 1;
                }

                if chunk_idx < chunks.len() {
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
                    frame.render_widget(mem_sparkline, chunks[chunk_idx]);
                    chunk_idx += 1;
                }

                if chunk_idx < chunks.len() {
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
                    frame.render_widget(conn_sparkline, chunks[chunk_idx]);
                }
            }
        }
    }
}

fn renderConfirmDialog(frame: &mut Frame, app: &App) {
    if let Some(proc) = app.getSelectedProcess() {
        let is_docker = proc.isDockerProcess();
        let area = centeredRect(50, 35, frame.area());

        frame.render_widget(Clear, area);

        let mut text = vec![
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
        ];

        if is_docker {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "⚠ Docker process detected",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            if !app.force_kill {
                text.push(Line::from(Span::styled(
                    "Press F to enable force mode first.",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                text.push(Line::from(Span::styled(
                    "Force mode active — proceed with caution!",
                    Style::default().fg(Color::Red),
                )));
            }
        }

        text.push(Line::from(""));
        text.push(Line::from(""));

        if is_docker && !app.force_kill {
            text.push(Line::from(vec![
                Span::styled("[F]", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" Force mode  ", Style::default().fg(Color::Yellow)),
                Span::styled("[N]", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" Cancel", Style::default().fg(Color::Green)),
            ]));
        } else {
            text.push(Line::from(vec![
                Span::styled("[Y]", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" Yes  ", Style::default().fg(Color::Red)),
                Span::styled("[N]", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" No", Style::default().fg(Color::Green)),
            ]));
        }

        let border_color = if is_docker && !app.force_kill {
            Color::Yellow
        } else {
            Color::Red
        };

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
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

    let table_width = chunks[1].width as usize;
    let fixed: usize = 12 + 20 + 8 + 2 + 4;
    let flex = table_width.saturating_sub(fixed);
    let path_w = (flex / 2).max(15);
    let cmd_w  = flex.saturating_sub(path_w).max(15);

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

        let path_trunc = truncateWithEllipsis(&proj.path, path_w);
        let cmd_trunc  = truncateWithEllipsis(&proj.run_command, cmd_w);

        Row::new(vec![
            Cell::from(status),
            Cell::from(proj.name.clone()).style(Style::default().fg(
                if i == app.projects_selected { Color::Black } else { Color::Cyan }
            )),
            Cell::from(format!(":{}", proj.port)).style(Style::default().fg(
                if i == app.projects_selected { Color::Black } else { Color::Yellow }
            )),
            Cell::from(path_trunc),
            Cell::from(cmd_trunc),
        ]).style(style)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Length(path_w as u16),
            Constraint::Min(cmd_w as u16),
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
