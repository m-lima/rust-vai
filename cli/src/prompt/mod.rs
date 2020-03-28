#![allow(unused_must_use)]

use vai_core as core;

mod action;
mod buffer;
mod completions;
mod navigation;
mod suggester;
mod terminal;

pub(super) struct Context {
    buffer: buffer::Buffer,
    suggester: suggester::Suggester,
    completions: Option<completions::Completions>,
}

fn new(buffer: buffer::Buffer, suggestions: Vec<String>) -> Context {
    let mut context = Context {
        buffer,
        suggester: suggester::new(suggestions),
        completions: None,
    };
    context.suggester.update(&context.buffer);
    context
}

impl Context {
    fn edit(&mut self, action: &action::EditAction) {
        self.buffer.edit(&action);
        self.suggester.update(&self.buffer);
        self.completions = None;
    }

    fn move_cursor(&mut self, scope: &action::Scope) {
        if self.buffer.at_end() {
            match scope {
                action::Scope::Forward | action::Scope::ForwardAll => {
                    self.buffer.write_str(&self.suggester.take());
                }
                action::Scope::ForwardWord => {
                    self.buffer.write_str(&self.suggester.take_next_word());
                }
                _ => {}
            }
        }
        self.buffer.move_cursor(&scope)
    }

    fn complete<S>(&mut self, direction: &action::Direction, supplier: S)
    where
        S: Fn() -> Vec<String>,
    {
        if let Some(completions) = self.completions.as_mut() {
            use action::Direction;
            if let Some(completion) = match direction {
                Direction::Down => completions.select_down(),
                Direction::Up => completions.select_up(),
            } {
                self.buffer.set_str(completion);
                self.suggester.update(&self.buffer);
            }
        } else {
            self.completions = Some(completions::new(supplier()));
        }
    }
}

fn read_query(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    buffer: buffer::Buffer,
    target: &str,
) {
    let executor = executors.find(target).unwrap();
    let mut context = new(buffer, executor.history().unwrap_or_else(|_| vec![]));
    terminal.set_prompt(&Some(target));

    loop {
        use action::Action;

        terminal.print(&context);

        match action::read() {
            Action::Noop => continue,
            Action::Exit => return,
            Action::Execute => {
                if let Err(e) = executor.execute(&context.buffer.data()) {
                    terminal::fatal(e);
                } else {
                    return;
                }
            }
            Action::Cancel => {
                return read_target(terminal, executors, buffer::from(target, context.buffer))
            }
            Action::Complete(direction) => {
                let query = context.buffer.data();
                let supplier = move || {
                    let mut completions = executor
                        .fuzzy_history(&query, 10)
                        .unwrap_or_else(|_| vec![]);
                    completions.extend(executor.suggest(&query).unwrap_or_else(|_| vec![]));
                    completions
                };
                context.complete(&direction, supplier);
            }
            Action::Edit(action) => context.edit(&action),
            Action::MoveCursor(scope) => context.move_cursor(&scope),
        }
    }
}

fn read_target(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    buffer: buffer::Buffer,
) {
    let mut context = new(
        buffer,
        executors
            .list_targets()
            .iter()
            .map(|suggestion| String::from(*suggestion))
            .collect(),
    );
    terminal.set_prompt(&None);

    loop {
        use action::Action;

        terminal.print(&context);

        match action::read() {
            Action::Noop | Action::Cancel => continue,
            Action::Execute => {
                if let Some((target, buffer)) = context.buffer.extract_first_word() {
                    if executors.find(&target).is_some() {
                        return read_query(terminal, executors, buffer, &target);
                    } else {
                        terminal.print_error("Invalid target");
                    }
                }
            }
            Action::Exit => return,
            Action::Complete(direction) => context.complete(&direction, || {
                executors
                    .list_targets()
                    .iter()
                    .map(|target| String::from(*target))
                    .collect::<Vec<_>>()
            }),
            Action::Edit(action) => context.edit(&action),
            Action::MoveCursor(scope) => context.move_cursor(&scope),
        }
    }
}

pub(super) fn run() {
    let terminal = terminal::new();

    let executors = match core::executors::load_default() {
        Ok(executors) => executors,
        Err(e) => terminal::fatal(e),
    };

    read_target(terminal, executors, buffer::new());
}
