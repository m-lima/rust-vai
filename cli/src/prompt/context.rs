use vai_core as core;

pub(super) enum UpgradeResult {
    Done,
    Continue,
    Error(String),
}

pub(super) struct Context {
    executors: core::executors::Executors,
    target: Option<String>,
    buffer: super::buffer::Buffer,
    suggester: super::suggester::Suggester,
    completions: Option<super::completions::Completions>,
}

pub(super) fn new() -> Context {
    let executors = match core::executors::load_default() {
        Ok(executors) => executors,
        Err(e) => super::terminal::fatal(e),
    };

    let suggester = target_suggester(&executors);

    Context {
        executors,
        target: None,
        buffer: super::buffer::new(),
        suggester,
        completions: None,
    }
}

fn target_suggester(executors: &core::executors::Executors) -> super::suggester::Suggester {
    super::suggester::new(
        executors
            .list_targets()
            .iter()
            .map(|suggestion| String::from(*suggestion))
            .collect(),
    )
}

impl Context {
    pub(super) fn target(&self) -> &Option<String> {
        &self.target
    }

    pub(super) fn buffer(&self) -> &super::buffer::Buffer {
        &self.buffer
    }

    pub(super) fn suggester(&self) -> &super::suggester::Suggester {
        &self.suggester
    }

    pub(super) fn completions(&self) -> &Option<super::completions::Completions> {
        &self.completions
    }

    pub(super) fn upgrade(&mut self) -> UpgradeResult {
        self.completions = None;

        if let Some(target) = &self.target {
            match self
                .executors
                .find(target)
                .unwrap()
                .execute(&self.buffer.data())
            {
                Ok(_) => UpgradeResult::Done,
                Err(e) => UpgradeResult::Error(e.to_string()),
            }
        } else {
            let mut old_buffer = self
                .buffer
                .data()
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<_>>();

            if old_buffer.is_empty() {
                UpgradeResult::Continue
            } else {
                let target = old_buffer.remove(0);

                if let Some(executor) = self.executors.find(&target) {
                    self.buffer = if old_buffer.is_empty() {
                        super::buffer::new()
                    } else {
                        use joinery::Joinable;
                        super::buffer::from(&old_buffer.join_with(' ').to_string())
                    };
                    self.suggester =
                        super::suggester::new(executor.history().unwrap_or_else(|_| vec![]));
                    self.target = Some(target);
                    UpgradeResult::Continue
                } else {
                    UpgradeResult::Error(String::from("Invalid target"))
                }
            }
        }
    }

    pub(super) fn downgrade(&mut self) {
        self.completions = None;

        if let Some(target) = &self.target {
            let suggester = target_suggester(&self.executors);

            // TODO move cursor to where `target` ends
            let mut target = target.clone();
            target.push(' ');
            target.push_str(&self.buffer.data());

            self.target = None;
            self.buffer.set_str(&target);
            self.suggester = suggester;
        }
    }

    pub(super) fn complete(&mut self, direction: &super::action::Direction) {
        self.buffer.write_str(&self.suggester.take());

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
            self.completions = Some(super::completions::new(
                if let Some(target) = &self.target {
                    let executor = self.executors.find(&target).unwrap();
                    let query = self.buffer.data();
                    let mut completions = executor
                        .fuzzy_history(&query, 10)
                        .unwrap_or_else(|_| vec![]);
                    completions.extend(executor.suggest(&query).unwrap_or_else(|_| vec![]));
                    completions
                } else {
                    self.executors
                        .list_targets()
                        .iter()
                        .map(|target| String::from(*target))
                        .collect()
                },
            ));
        }
    }

    pub(super) fn edit(&mut self, action: &super::action::EditAction) {
        self.buffer.edit(&action);
        self.suggester.update(&self.buffer);
        self.completions = None;
    }

    pub(super) fn cursor(&mut self, scope: &super::action::Scope) {
        use super::action::Scope;
        if self.buffer.at_end() {
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
}
