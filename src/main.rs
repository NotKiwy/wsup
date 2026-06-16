mod core;
mod ui;
mod utils;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use ui::{App, renderUi, ViewMode, SortMode};

#[derive(Parser, Debug)]
#[command(name = "wsup")]
#[command(about = "TUI localhost process manager")]
#[command(version)]
#[command(disable_help_flag = true)]
#[command(disable_colored_help = true)]
struct Args {
    #[arg(short, long, value_name = "MODE", help = "Sort by: port, cpu, memory, connections, name")]
    sort: Option<String>,

    #[arg(short, long, value_name = "QUERY", alias = "search", help = "Filter processes by name, port, or command")]
    filter: Option<String>,

    #[arg(short = 'k', long, value_name = "PORT", help = "Kill process on specified port")]
    kill: Option<u16>,

    #[arg(short, long, action = clap::ArgAction::HelpShort, help = "Print help")]
    help: Option<bool>,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    if let Some(port) = args.kill {
        use crate::core::{getLocalhostProcesses, killProcess};
        use crossterm::style::{Color, Stylize};

        let processes = getLocalhostProcesses();
        let target = processes.iter().find(|p| p.port == port);

        match target {
            Some(proc) => {
                println!("Killing process {} {} on port {}",
                    proc.name.clone().with(Color::Green),
                    format!("(PID: {})", proc.pid).with(Color::DarkGrey),
                    format!(":{}", port).with(Color::Cyan)
                );
                match killProcess(proc.pid) {
                    Ok(_) => {
                        println!("{}", "✓ Process killed successfully".with(Color::Green));
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("{} {}", "✗ Failed to kill process:".with(Color::Red), e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                eprintln!("{} {}",
                    "✗ No process found on port".with(Color::Red),
                    format!(":{}", port).with(Color::Cyan)
                );
                std::process::exit(1);
            }
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    if let Some(sort) = args.sort {
        app.sort_mode = match sort.to_lowercase().as_str() {
            "cpu" => SortMode::Cpu,
            "memory" | "mem" => SortMode::Memory,
            "connections" | "conn" => SortMode::Connections,
            "name" => SortMode::Name,
            _ => SortMode::Port,
        };
    }

    if let Some(filter) = args.filter {
        app.search_query = filter;
    }

    app.refreshProcesses();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        let size = terminal.size()?;
        let visible_rows = size.height.saturating_sub(5) as usize;

        if app.shouldAutoRefresh() {
            app.refreshProcesses();
        }

        terminal.draw(|f| renderUi(f, app))?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.search_mode {
                        match key.code {
                            KeyCode::Esc => {
                                app.toggleSearch();
                            }
                            KeyCode::Enter => {
                                app.toggleSearch();
                            }
                            KeyCode::Backspace => {
                                app.searchBackspace();
                            }
                            KeyCode::Char(c) => {
                                app.searchInput(c);
                            }
                            _ => {}
                        }
                    } else {
                        match app.view_mode {
                            ViewMode::List => {
                                match key.code {
                                    KeyCode::Char('q') => app.quit(),
                                    KeyCode::Char('/') => {
                                        app.toggleSearch();
                                    }
                                    KeyCode::Char('r') => {
                                        app.refreshProcesses();
                                        app.clearStatus();
                                    }
                                    KeyCode::Char('s') => {
                                        app.cycleSort();
                                    }
                                    KeyCode::Char('o') => {
                                        app.toggleSortOrder();
                                    }
                                    KeyCode::Char('x') | KeyCode::Char('d') => {
                                        app.showConfirmKill();
                                    }
                                    KeyCode::Enter => {
                                        app.showDetail();
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        app.next();
                                        app.adjustScroll(visible_rows);
                                        app.clearStatus();
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        app.previous();
                                        app.adjustScroll(visible_rows);
                                        app.clearStatus();
                                    }
                                    _ => {}
                                }
                            }
                            ViewMode::Projects => {
                                match key.code {
                                    KeyCode::Char('q') => app.quit(),
                                    KeyCode::Esc => app.backToList(),
                                    KeyCode::Char(' ') => {
                                        app.toggleProject();
                                    }
                                    KeyCode::Char('d') => {
                                        app.deployProject();
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        app.projectsNext();
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        app.projectsPrevious();
                                    }
                                    _ => {}
                                }
                            }
                            ViewMode::Detail => {
                                match key.code {
                                    KeyCode::Esc => app.backToList(),
                                    KeyCode::Char('x') => {
                                        app.showConfirmKill();
                                    }
                                    _ => {}
                                }
                            }
                            ViewMode::ConfirmKill => {
                                match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        app.killSelected();
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                        app.backToList();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
