#![allow(unused_must_use)]

use vai_core as core;

use std::io::Write;

static PROMPT_START: u16 = 6;

enum Action {
    Edit(EditAction),
    MoveCursor(Scope),
    Execute,
    Cancel,
    Complete,
    Noop,
}

enum EditAction {
    Write(char),
    Delete(Scope),
}

enum Scope {
    Back,
    Forward,
    BackWord,
    ForwardWord,
    BackAll,
    ForwardAll,
    All,
    WordAll,
}

struct Suggester<'a> {
    targets: Vec<&'a String>,
    data: String,
    last_size: usize,
}

struct Buffer<'a> {
    position: usize,
    data: Vec<char>,
    suggestion: Suggester<'a>,
}

impl<'a> Buffer<'a> {
    fn new(executors: &'a core::executors::Executors) -> Self {
        Self {
            position: 0,
            data: Vec::new(),
            suggestion: Suggester {
                targets: executors.list_targets(),
                data: String::new(),
                last_size: 0,
            },
        }
    }

    // Allowed because it is guarded in the `write` method
    #[allow(clippy::cast_possible_truncation)]
    fn position(&self) -> u16 {
        self.position as u16
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

    fn generate_suggestion(&mut self) {
        if self.position != self.data.len()
            || self.position == 0
            || self.data[self.position - 1].is_whitespace()
            || self.suggestion.last_size == self.data.len()
        {
            return;
        }

        let start = self.find_previous_word();
        let last_word = self.data[start..self.position].iter().collect::<String>();
        self.suggestion.last_size = self.data.len();

        if let Some(suggestion) = self
            .suggestion
            .targets
            .iter()
            .find(|target| target.starts_with(&last_word))
        {
            self.suggestion.data = String::from(&suggestion[last_word.len()..suggestion.len()]);
        } else {
            self.suggestion.data.clear();
        }
    }

    fn write(&mut self, c: char) {
        if self.data.len() < usize::from(u16::max_value() - PROMPT_START) {
            self.data.insert(self.position, c);
            self.position += 1;
        }
    }

    fn delete(&mut self, scope: &Scope) {
        match scope {
            Scope::Back => {
                if self.position > 0 {
                    self.data.remove(self.position - 1);
                    self.position -= 1;
                }
            }
            Scope::Forward => {
                if self.position < self.data.len() {
                    self.data.remove(self.position);
                }
            }
            Scope::BackWord => {
                let index = self.find_previous_word();
                self.data.drain(index..self.position);
                self.position = index;
            }
            Scope::ForwardWord => {
                let index = self.find_next_word();
                self.data.drain(self.position..index);
            }
            Scope::BackAll => {
                self.data.drain(0..self.position);
                self.position = 0;
            }
            Scope::ForwardAll => {
                self.data.drain(self.position..self.data.len());
            }
            Scope::All => {
                self.data.clear();
                self.position = 0;
            }
            Scope::WordAll => {
                let start = self.find_previous_word_end();
                let end = self.find_next_word();
                self.data.drain(start + 1..end);
                self.data[start] = ' ';
                self.position = start;
            }
        }
    }

    fn edit(&mut self, action: &EditAction) {
        match action {
            EditAction::Write(c) => self.write(*c),
            EditAction::Delete(scope) => self.delete(scope),
        }

        self.generate_suggestion();
    }

    fn move_cursor(&mut self, scope: &Scope) {
        match scope {
            Scope::Back => {
                self.position -= 1;
            }
            Scope::Forward => {
                if self.position < self.data.len() {
                    self.position += 1;
                } else {
                    let mut suggestion = String::new();
                    std::mem::swap(&mut self.suggestion.data, &mut suggestion);
                    suggestion.chars().for_each(|c| self.write(c));
                }
            }
            Scope::BackWord => {
                self.position = self.find_previous_word();
            }
            Scope::ForwardWord => {
                self.position = self.find_next_word();
            }
            Scope::BackAll => {
                self.position = 0;
            }
            Scope::ForwardAll => {
                self.position = self.data.len();
            }
            Scope::WordAll | Scope::All => unreachable!("No available cursor movements for these"),
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
            crossterm::event::KeyCode::Backspace => Action::Edit(EditAction::Delete(Scope::Back)),
            crossterm::event::KeyCode::Delete => Action::Edit(EditAction::Delete(Scope::Forward)),
            crossterm::event::KeyCode::Right => Action::MoveCursor(Scope::Forward),
            crossterm::event::KeyCode::Left => Action::MoveCursor(Scope::Back),
            crossterm::event::KeyCode::Home => Action::MoveCursor(Scope::BackAll),
            crossterm::event::KeyCode::End => Action::MoveCursor(Scope::ForwardAll),
            crossterm::event::KeyCode::Char(c) => {
                if control(&e) {
                    match c {
                        'm' => Action::Execute,
                        'c' => Action::Cancel,

                        'b' => Action::MoveCursor(Scope::Back),
                        'f' => Action::MoveCursor(Scope::Forward),
                        'a' => Action::MoveCursor(Scope::BackAll),
                        'e' => Action::MoveCursor(Scope::ForwardAll),

                        'j' => Action::Edit(EditAction::Delete(Scope::BackWord)),
                        'k' => Action::Edit(EditAction::Delete(Scope::ForwardWord)),
                        'h' => Action::Edit(EditAction::Delete(Scope::BackAll)),
                        'l' => Action::Edit(EditAction::Delete(Scope::ForwardAll)),
                        'w' => Action::Edit(EditAction::Delete(Scope::WordAll)),
                        'u' => Action::Edit(EditAction::Delete(Scope::All)),
                        _ => Action::Noop,
                    }
                } else if alt(&e) {
                    match c {
                        'b' => Action::MoveCursor(Scope::BackWord),
                        'f' => Action::MoveCursor(Scope::ForwardWord),
                        _ => Action::Noop,
                    }
                } else {
                    Action::Edit(EditAction::Write(c))
                }
            }
            _ => Action::Noop,
        },
        Ok(_) => Action::Noop,
        Err(e) => process_error(e),
    }
}

fn execute(buffer: Buffer<'_>) -> ! {
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

fn complete() {}

pub(super) fn run() -> ! {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Green),
        crossterm::style::Print("vai"),
        crossterm::style::ResetColor,
        crossterm::style::Print("> "),
    );

    let executors = match core::executors::load_default() {
        Ok(executors) => executors,
        Err(e) => process_error(e),
    };

    let mut buffer = Buffer::new(&executors);

    crossterm::terminal::enable_raw_mode();

    loop {
        match read_action() {
            Action::Noop => continue,
            Action::Execute => execute(buffer),
            Action::Cancel => exit(0),
            Action::Complete => complete(),
            Action::Edit(action) => buffer.edit(&action),
            Action::MoveCursor(scope) => buffer.move_cursor(&scope),
        }

        let cursor = PROMPT_START + buffer.position();
        let buffer_string = buffer.data.iter().collect::<String>();

        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveToColumn(PROMPT_START),
            crossterm::style::ResetColor,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            crossterm::style::Print(buffer_string),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
            crossterm::style::Print(&buffer.suggestion.data),
            crossterm::style::ResetColor,
            crossterm::cursor::MoveToColumn(cursor),
        );
    }
}
