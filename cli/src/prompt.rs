#![allow(unused_must_use)]

use std::io::Write;

enum Action {
    Execute,
    Cancel,
    Complete,
    Write(char),
    Noop,

    DeleteBack,
    DeleteForward,
    DeleteBackWord,
    DeleteForwardWord,
    DeleteBackAll,
    DeleteForwardAll,
    DeleteAll,
    DeleteWordAll,

    MoveBack,
    MoveForward,
    MoveBackWord,
    MoveForwardWord,
    MoveBackAll,
    MoveForwardAll,
}

struct Buffer {
    position: usize,
    data: String,
}

impl Buffer {
    fn new() -> Self {
        Self {
            position: 0,
            data: String::new(),
        }
    }

    // Allowed for now, since it's still in development
    #[allow(clippy::match_same_arms)]
    fn process_action(&mut self, action: &Action) {
        match action {
            Action::Execute | Action::Cancel | Action::Complete | Action::Noop => {}
            Action::Write(c) => {
                self.data.insert(self.position, *c);
                self.position += 1;
            }
            Action::DeleteBack => {
                if self.position > 0 {
                    self.data.remove(self.position - 1);
                    self.position -= 1;
                }
            }
            Action::DeleteForward => {
                if self.position < self.data.len() {
                    self.data.remove(self.position);
                }
            }
            Action::DeleteBackWord => {
                // if let Some(index) = self.data[0..self.position].rfind(' ') {
                //     self.data = String::from(&self.data[index..self.data.len()]);
                //     self.position = index;
                // } else {
                //     self.data = String::from(&self.data[self.position..self.data.len()]);
                //     self.position = 0;
                // }
            }
            Action::DeleteForwardWord => {}
            Action::DeleteBackAll => {
                self.data = String::from(&self.data[self.position..self.data.len()]);
                self.position = 0;
            }
            Action::DeleteForwardAll => {
                self.data = String::from(&self.data[0..self.position]);
            }
            Action::DeleteAll => {
                self.data = String::new();
                self.position = 0;
            }
            Action::DeleteWordAll => {}
            Action::MoveBack => {
                self.position -= 1;
            }
            Action::MoveForward => {
                self.position += 1;
            }
            Action::MoveBackWord => {
                // if let Some(index) = self.data[0..self.position].rfind(' ') {
                //     self.position = index;
                // } else {
                //     self.position = 0;
                // }
            }
            Action::MoveForwardWord => {}
            Action::MoveBackAll => {
                self.position = 0;
            }
            Action::MoveForwardAll => {
                self.position = self.data.len();
            }
        }
    }
}

fn control(e: &crossterm::event::KeyEvent) -> bool {
    e.modifiers == crossterm::event::KeyModifiers::CONTROL
}

fn process_error<M: std::fmt::Display>(message: M) -> ! {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::Print("\n"),
        crossterm::cursor::MoveToColumn(0),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
        crossterm::style::Print("Error: "),
        crossterm::style::ResetColor,
        crossterm::style::Print(message.to_string()),
    );
    exit(-1);
}

fn exit(code: i32) -> ! {
    crossterm::terminal::disable_raw_mode();
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::ResetColor,
        crossterm::style::Print('\n'),
    );

    std::process::exit(code);
}

fn read_action(escape: &mut bool) -> Action {
    let was_escaped = *escape;
    *escape = false;

    match crossterm::event::read() {
        Ok(crossterm::event::Event::Key(e)) => match e.code {
            crossterm::event::KeyCode::Enter => Action::Execute,
            crossterm::event::KeyCode::Tab => Action::Complete,
            crossterm::event::KeyCode::Backspace => Action::DeleteBack,
            crossterm::event::KeyCode::Delete => Action::DeleteForward,
            crossterm::event::KeyCode::Right => Action::MoveForward,
            crossterm::event::KeyCode::Left => Action::MoveBack,
            crossterm::event::KeyCode::Home => Action::MoveBackAll,
            crossterm::event::KeyCode::End => Action::MoveForwardAll,
            crossterm::event::KeyCode::Esc => {
                if !was_escaped {
                    *escape = true;
                }
                Action::Noop
            }
            crossterm::event::KeyCode::Char(c) => {
                if control(&e) {
                    match c {
                        'm' => Action::Execute,
                        'c' => Action::Cancel,

                        'b' => Action::MoveBack,
                        'f' => Action::MoveForward,
                        'a' => Action::MoveBackAll,
                        'e' => Action::MoveForwardAll,

                        'j' => Action::DeleteBackWord,
                        'k' => Action::DeleteForwardWord,
                        'h' => Action::DeleteBackAll,
                        'l' => Action::DeleteForwardAll,
                        'w' => Action::DeleteWordAll,
                        'u' => Action::DeleteAll,
                        _ => Action::Noop,
                    }
                } else if was_escaped {
                    std::fs::write(
                        std::path::Path::new("/tmp/vai_output"),
                        "Processsing escaped\n",
                    );
                    match c {
                        'b' => Action::MoveBackWord,
                        'f' => Action::MoveForwardWord,
                        _ => Action::Noop,
                    }
                } else {
                    Action::Write(c)
                }
            }
            _ => Action::Noop,
        },
        Ok(_) => Action::Noop,
        Err(e) => process_error(e),
    }
}

fn execute(buffer: &Buffer) -> ! {
    let args = buffer
        .data
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<_>>();

    if args.len() < 2 {
        exit(-1);
    }

    match super::execute(args) {
        Ok(()) => exit(0),
        Err(e) => process_error(e),
    }
}

// Allowed while still in development. String might not be used after all
#[allow(clippy::cast_possible_truncation)]
pub(super) fn run(name: &str) -> ! {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
        crossterm::style::Print(&name),
        crossterm::style::ResetColor,
        crossterm::style::Print("> "),
    );

    let start = crossterm::cursor::position().map_or_else(|_| (name.len() + 2) as u16, |p| p.0) + 1;

    let mut buffer = Buffer::new();
    let mut escape = false;

    crossterm::terminal::enable_raw_mode();
    loop {
        match read_action(&mut escape) {
            Action::Noop => continue,
            Action::Execute => execute(&buffer),
            Action::Cancel => exit(0),
            Action::Complete => {}
            action => buffer.process_action(&action),
        }

        let cursor = start + buffer.position as u16;

        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToColumn(start),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            crossterm::style::Print(&buffer.data),
            crossterm::cursor::MoveToColumn(cursor),
        );
    }
}
