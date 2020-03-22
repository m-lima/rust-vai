pub(super) struct Buffer {
    max_size: usize,
    data: Vec<char>,
    position: usize,
}

pub(super) fn new(max_size: u16) -> Buffer {
    Buffer {
        max_size: usize::from(max_size),
        data: Vec::new(),
        position: 0,
    }
}

impl Buffer {
    // Allowed because it is guarded in the `write` method
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn position(&self) -> u16 {
        self.position as u16
    }

    pub(super) fn at_end(&self) -> bool {
        self.position == self.data.len()
    }

    pub(super) fn data(&self) -> String {
        self.data.iter().collect()
    }

    pub(super) fn data_raw(&self) -> &Vec<char> {
        &self.data
    }

    pub(super) fn write_str(&mut self, string: &str) {
        string.chars().for_each(|c| self.write(c));
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
                let index = super::navigation::previous_word(self.position, &self.data);
                self.data.drain(index..self.position);
                self.position = index;
            }
            ForwardWord => {
                let index = super::navigation::next_word(self.position, &self.data);
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
                let start = super::navigation::previous_word_end(self.position, &self.data);
                let end = super::navigation::next_word(self.position, &self.data);
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
    }

    pub(super) fn move_cursor(&mut self, scope: &super::action::Scope) {
        use super::action::Scope::*;
        match scope {
            Back => {
                if self.position > 0 {
                    self.position -= 1;
                }
            }
            Forward => {
                if self.position < self.data.len() {
                    self.position += 1;
                }
            }
            BackWord => {
                self.position = super::navigation::previous_word(self.position, &self.data);
            }
            ForwardWord => {
                self.position = super::navigation::next_word(self.position, &self.data);
            }
            BackAll => {
                self.position = 0;
            }
            ForwardAll => {
                if self.position < self.data.len() {
                    self.position = self.data.len();
                }
            }
            WordAll | All => {}
        }
    }
}
