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
    data: Vec<char>,
}

impl Buffer {
    fn new() -> Self {
        Self {
            position: 0,
            data: Vec::new(),
        }
    }

    fn find_next_word(&self) -> usize {
        let end = self.data.len();
        if self.position == end {
            self.position
        } else {
            let mut index = self.position;

            while index < end && !self.data[index].is_whitespace() {
                index += 1;
            }

            while index < end && self.data[index].is_whitespace() {
                index += 1;
            }

            index
        }
    }

    fn find_previous_word(&self) -> usize {
        if self.position == 0 {
            self.position
        } else {
            let mut index = self.position - 1;

            while index > 0 && self.data[index].is_whitespace() {
                index -= 1;
            }

            while index > 0 && !self.data[index - 1].is_whitespace() {
                index -= 1;
            }

            index
        }
    }

    fn find_previous_word_end(&self) -> usize {
        if self.position == 0 {
            self.position
        } else {
            let mut index = self.position - 1;

            while index > 0 && !self.data[index].is_whitespace() {
                index -= 1;
            }

            while index > 0 && self.data[index - 1].is_whitespace() {
                index -= 1;
            }

            index
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
            Action::DeleteBackWord => {
                let index = self.find_previous_word();
                self.data.drain(index..self.position);
                self.position = index;
            }
            Action::DeleteForwardWord => {
                let index = self.find_next_word();
                self.data.drain(self.position..index);
            }
            Action::DeleteBackAll => {
                self.data.drain(0..self.position);
                self.position = 0;
            }
            Action::DeleteForwardAll => {
                self.data.drain(self.position..self.data.len());
            }
            Action::DeleteAll => {
                self.data.clear();
                self.position = 0;
            }
            Action::DeleteWordAll => {
                let start = self.find_previous_word_end();
                let end = self.find_next_word();
                self.data.drain(start + 1..end);
                self.data[start] = ' ';
                self.position = start;
            }
            Action::MoveBack => {
                self.position -= 1;
            }
            Action::MoveForward => {
                self.position += 1;
            }
            Action::MoveBackWord => {
                self.position = self.find_previous_word();
            }
            Action::MoveForwardWord => {
                self.position = self.find_next_word();
            }
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

fn read_action() -> Action {
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
                } else if alt(&e) {
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

fn execute(buffer: Buffer) -> ! {
    let args = buffer
        .data
        .into_iter()
        .collect::<String>()
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

    crossterm::terminal::enable_raw_mode();

    loop {
        match read_action() {
            Action::Noop => continue,
            Action::Execute => execute(buffer),
            Action::Cancel => exit(0),
            Action::Complete => {}
            action => buffer.process_action(&action),
        }

        let cursor = start + buffer.position as u16;
        let buffer_string = buffer.data.iter().collect::<String>();

        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToColumn(start),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            crossterm::style::Print(buffer_string),
            crossterm::cursor::MoveToColumn(cursor),
        );
    }
}
