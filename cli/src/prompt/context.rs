pub(super) struct Context {
    buffer: super::buffer::Buffer,
    suggester: super::suggester::Suggester,
    completions: Option<super::completions::Completions>,
}

pub(super) fn new(buffer: super::buffer::Buffer, suggestions: Vec<String>) -> Context {
    let mut context = Context {
        buffer,
        suggester: super::suggester::new(suggestions),
        completions: None,
    };
    context.suggester.update(&context.buffer);
    context
}

impl Context {
    pub(super) fn buffer(&self) -> &super::buffer::Buffer {
        &self.buffer
    }

    pub(super) fn suggester(&self) -> &super::suggester::Suggester {
        &self.suggester
    }

    pub(super) fn completions(&self) -> &Option<super::completions::Completions> {
        &self.completions
    }

    pub(super) fn edit(&mut self, action: &super::action::EditAction) {
        self.buffer.edit(&action);
        self.suggester.update(&self.buffer);
        self.completions = None;
    }

    pub(super) fn move_cursor(&mut self, scope: &super::action::Scope) {
        if self.buffer.at_end() {
            use super::action::Scope;
            match scope {
                Scope::Forward | Scope::ForwardAll => {
                    self.buffer.write_str(&self.suggester.take());
                }
                Scope::ForwardWord => {
                    self.buffer.write_str(&self.suggester.take_next_word());
                }
                _ => {}
            }
        }
        self.buffer.move_cursor(&scope)
    }

    pub(super) fn complete<S>(&mut self, direction: &super::action::Direction, supplier: S)
    where
        S: Fn() -> Vec<String>,
    {
        if let Some(completions) = self.completions.as_mut() {
            use super::action::Direction;
            if let Some(completion) = match direction {
                Direction::Down => completions.select_down(),
                Direction::Up => completions.select_up(),
            } {
                self.buffer.set_str(completion);
                self.suggester.update(&self.buffer);
            }
        } else {
            self.completions = Some(super::completions::new(supplier()));
        }
    }
}
