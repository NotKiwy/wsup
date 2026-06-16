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
#[command(long_about = "Monitor and manage processes running on localhost ports")]
#[command(version)]
struct Args {
    #[arg(short, long, value_name = "MODE")]
    sort: Option<String>,

    #[arg(short, long, value_name = "QUERY", alias = "search")]
    filter: Option<String>,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

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
