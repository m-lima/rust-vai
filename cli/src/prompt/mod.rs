#![allow(unused_must_use)]

use vai_core as core;

mod action;
mod buffer;
mod completions;
mod context;
mod navigation;
mod suggester;
mod terminal;

fn read_query(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    buffer: buffer::Buffer,
    target: &str,
) {
    let executor = executors.find(target).unwrap();
    let mut context = context::new(buffer, executor.history().unwrap_or_else(|_| vec![]));
    terminal.set_prompt(Some(target));

    loop {
        use action::Action;

        terminal.print(&context);

        match action::read() {
            Action::Noop => continue,
            Action::Exit => return,
            Action::Execute => {
                if let Err(e) = executor.execute(&context.buffer().data()) {
                    terminal::fatal(e);
                } else {
                    return;
                }
            }
            Action::Cancel => {
                return read_target(terminal, executors, buffer::from(target, context.buffer()))
            }
            Action::Complete(direction) => {
                let query = context.buffer().data();
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
    let mut context = context::new(
        buffer,
        executors
            .list_targets()
            .iter()
            .map(|suggestion| String::from(*suggestion))
            .collect(),
    );
    terminal.set_prompt(None);

    loop {
        use action::Action;

        terminal.print(&context);

        match action::read() {
            Action::Noop | Action::Cancel => continue,
            Action::Execute => {
                if let Some((target, buffer)) = context.buffer().extract_first_word() {
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
