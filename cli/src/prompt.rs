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
            Action::DeleteBackWord => {}
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
            Action::MoveBackWord => {}
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

fn alt(e: &crossterm::event::KeyEvent) -> bool {
    e.modifiers == crossterm::event::KeyModifiers::ALT
}

fn process_error<M: std::fmt::Display>(message: M) -> ! {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
        crossterm::style::Print("\nError: "),
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

fn read_action() -> Action {
    match crossterm::event::read() {
        Ok(crossterm::event::Event::Key(e)) => match e.code {
            crossterm::event::KeyCode::Enter => Action::Execute,
            crossterm::event::KeyCode::Tab => Action::Complete,
            crossterm::event::KeyCode::Backspace => {
                if alt(&e) {
                    Action::DeleteBackWord
                } else {
                    Action::DeleteBack
                }
            }
            crossterm::event::KeyCode::Delete => {
                if alt(&e) {
                    Action::DeleteForwardWord
                } else {
                    Action::DeleteForward
                }
            }
            crossterm::event::KeyCode::Right => {
                if alt(&e) {
                    Action::MoveForwardWord
                } else {
                    Action::MoveForward
                }
            }
            crossterm::event::KeyCode::Left => {
                if alt(&e) {
                    Action::MoveBackWord
                } else {
                    Action::MoveBack
                }
            }
            crossterm::event::KeyCode::Home => Action::MoveBackAll,
            crossterm::event::KeyCode::End => Action::MoveForwardAll,
            crossterm::event::KeyCode::Char(c) => {
                if control(&e) {
                    match c {
                        'm' => Action::Execute,
                        'c' => Action::Cancel,
                        'f' => Action::MoveForward,
                        'b' => Action::MoveBack,
                        'a' => Action::MoveBackAll,
                        'e' => Action::MoveForwardAll,
                        'h' => Action::DeleteBack,
                        'd' => Action::DeleteForwardAll,
                        'l' => Action::DeleteBackAll,
                        'u' => Action::DeleteAll,
                        'w' => Action::DeleteWordAll,
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

pub(super) fn run(name: String) -> ! {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
        crossterm::style::Print(&name),
        crossterm::style::ResetColor,
        crossterm::style::Print("> "),
    );

    let start = crossterm::cursor::position()
        .map(|p| p.0)
        .unwrap_or_else(|_| (name.len() + 2) as u16)
        + 1;

    let mut buffer = Buffer::new();

    crossterm::terminal::enable_raw_mode();
    loop {
        match read_action() {
            Action::Noop => continue,
            Action::Execute => {}
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
