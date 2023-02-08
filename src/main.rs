pub mod file_editor;
pub mod task;
pub mod stateful_list;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use file_editor::save_file;
use task::Task;
use std::{error::Error, fs, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to file.
    #[arg(short, long)]
    path: String,
}

struct AppState {
    tasks: stateful_list::StatefulList<Task>,
    path: String,
}

impl AppState {
    fn new(tasks: Vec<Task>, path: String) -> Self {
        Self {
            tasks: stateful_list::StatefulList::with_items(tasks),
            path,
        }
    }
}

fn to_file(tasks: &Vec<Task>) -> String {
    let mut content = String::new();

    for task in tasks.iter() {
        content.push_str(
            format!(
                "[{}] {}\n",
                if task.completed { "x" } else { " " },
                task.content
            )
            .as_str(),
        );
    }

    content
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.path.is_empty() {
        panic!("Valid path is required.")
    }

    let tasks: Vec<Task> = fs::read_to_string(&args.path)
        .expect("Unable to open file")
        .trim()
        .lines()
        .map(|item| Task {
            completed: item.chars().nth(1).unwrap() == 'x',
            content: item[3..].trim().to_string(),
        })
        .collect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_state = AppState::new(tasks, args.path);
    let res = run_app(&mut terminal, app_state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app_state: AppState,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, &mut app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                // Exit keys
                KeyCode::Char('q') => return Ok(()),

                // task interaction keys
                KeyCode::Char(' ') | KeyCode::Enter => {
                    if let Some(index) = app_state.tasks.selected() {
                        app_state.tasks.items[index].select()
                    }
                }
                KeyCode::Char('S') => save_file(to_file(&app_state.tasks.items), &app_state.path)?,

                // Vertical movement keys
                KeyCode::Char('k') | KeyCode::Up => app_state.tasks.previous(),
                KeyCode::Char('j') | KeyCode::Down => app_state.tasks.next(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState) {
    let size = f.size();

    let create_block = |title: &str| {
        Block::default()
            .borders(Borders::ALL)
            .title(title.to_string())
    };

    let chunks = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    let tasks: Vec<ListItem> = app_state
        .tasks
        .items
        .iter()
        .map(|task| {
            ListItem::new(format!(
                "[{}] {}",
                if task.completed { "x" } else { " " },
                task.content
            ))
        })
        .collect();

    let tasks_list = List::new(tasks)
        .block(create_block("Checklist"))
        .highlight_style(Style::default().fg(Color::LightGreen))
        .start_corner(Corner::TopLeft);

    f.render_stateful_widget(tasks_list, chunks[0], &mut app_state.tasks.state)
}
