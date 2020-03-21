pub(super) struct Buffer<F: Fn(&str) -> Option<String>> {
    data: Vec<char>,
    suggestion: String,
    max_size: usize,
    position: usize,
    suggester: F,
}

pub(super) fn new<F: Fn(&str) -> Option<String>>(
    prompt_size: u16,
    data: Option<String>,
    suggester: F,
) -> Buffer<F> {
    let mut buffer = Buffer {
        max_size: usize::from(u16::max_value() - prompt_size),
        position: 0,
        data: Vec::new(),
        suggester,
        suggestion: String::new(),
    };

    if let Some(data) = data {
        data.chars().for_each(|c| buffer.write(c));
    }

    buffer
}

impl<F: Fn(&str) -> Option<String>> Buffer<F> {
    // Allowed because it is guarded in the `write` method
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn position(&self) -> u16 {
        self.position as u16
    }

    pub(super) fn suggestion(&self) -> &String {
        &self.suggestion
    }

    pub(super) fn data(&self) -> String {
        self.data.iter().collect()
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
        {
            return;
        }

        let start = self.find_previous_word();
        let last_word = self.data[start..self.position].iter().collect::<String>();

        if let Some(suggestion) = (self.suggester)(&last_word) {
            self.suggestion = String::from(&suggestion[last_word.len()..suggestion.len()]);
        } else {
            self.suggestion.clear();
        }
    }

    fn write(&mut self, c: char) {
        if self.data.len() < self.max_size {
            self.data.insert(self.position, c);
            self.position += 1;
        }
    }

    fn delete(&mut self, scope: &super::action::Scope) {
        use super::action::Scope::*;
        match scope {
            Back => {
                if self.position > 0 {
                    self.data.remove(self.position - 1);
                    self.position -= 1;
                }
            }
            Forward => {
                if self.position < self.data.len() {
                    self.data.remove(self.position);
                }
            }
            BackWord => {
                let index = self.find_previous_word();
                self.data.drain(index..self.position);
                self.position = index;
            }
            ForwardWord => {
                let index = self.find_next_word();
                self.data.drain(self.position..index);
            }
            BackAll => {
                self.data.drain(0..self.position);
                self.position = 0;
            }
            ForwardAll => {
                self.data.drain(self.position..self.data.len());
            }
            All => {
                self.data.clear();
                self.position = 0;
            }
            WordAll => {
                let start = self.find_previous_word_end();
                let end = self.find_next_word();
                self.data.drain(start + 1..end);
                self.data[start] = ' ';
                self.position = start;
            }
        }
    }

    pub(super) fn edit(&mut self, action: &super::action::EditAction) {
        match action {
            super::action::EditAction::Write(c) => self.write(*c),
            super::action::EditAction::Delete(scope) => self.delete(scope),
        }

        self.generate_suggestion();
    }

    pub(super) fn move_cursor(&mut self, scope: &super::action::Scope) {
        use super::action::Scope::*;
        match scope {
            Back => {
                self.position -= 1;
            }
            Forward => {
                if self.position < self.data.len() {
                    self.position += 1;
                } else {
                    let mut suggestion = String::new();
                    std::mem::swap(&mut self.suggestion, &mut suggestion);
                    suggestion.chars().for_each(|c| self.write(c));
                }
            }
            BackWord => {
                self.position = self.find_previous_word();
            }
            ForwardWord => {
                self.position = self.find_next_word();
            }
            BackAll => {
                self.position = 0;
            }
            ForwardAll => {
                self.position = self.data.len();
            }
            WordAll | All => {}
        }
    }
}
