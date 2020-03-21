#![allow(unused_must_use)]

use vai_core as core;

mod action;
mod buffer;
mod terminal;

fn execute<F: Fn(&str) -> Option<String>>(target: String, buffer: &buffer::Buffer<F>) {
    let mut args = vec![target];
    buffer
        .data()
        .split_whitespace()
        .for_each(|arg| args.push(String::from(arg)));

    if let Err(e) = super::execute(args) {
        terminal::fatal(e);
    }
}

fn complete() {}

fn read_query(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    target: String,
    args: String,
) {
    let executor = if let Some(executor) = executors.find(&target) {
        executor
    } else {
        terminal.print_error("Invalid target");
        let mut buffer = target;
        if !args.is_empty() {
            buffer.push(' ');
            buffer.push_str(&args)
        };
        return read_target(terminal, executors, Some(buffer));
    };

    let mut buffer = buffer::new(
        terminal.prompt_size(),
        if args.is_empty() { None } else { Some(args) },
        |query| {
            executor
                .strict_history(query, 1)
                .ok()
                .and_then(|history| history.first().map(String::from))
        },
    );

    terminal.prompt(Some(&target));
    loop {
        use action::Action;

        terminal.print(&buffer);

        match action::read() {
            Action::Noop => continue,
            Action::Exit => return,
            Action::Execute => return execute(target, &buffer),
            Action::Cancel => return read_target(terminal, executors, Some(target)),
            Action::Complete => complete(),
            Action::Edit(action) => buffer.edit(&action),
            Action::MoveCursor(scope) => buffer.move_cursor(&scope),
        }

        terminal.clear_error();
    }
}

fn read_target(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    buffer: Option<String>,
) {
    // Allowed because I disagree with clippy's argument for readability
    #[allow(clippy::find_map)]
    let mut buffer = buffer::new(terminal.prompt_size(), buffer, |target| {
        executors
            .list_targets()
            .iter()
            .find(|suggestion| suggestion.starts_with(target))
            .map(|suggestion| String::from(*suggestion))
    });

    terminal.prompt(None);
    loop {
        use action::Action;

        terminal.print(&buffer);

        match action::read() {
            Action::Noop | Action::Cancel => continue,
            Action::Execute => {
                let mut data = buffer
                    .data()
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>();
                if data.is_empty() {
                    continue;
                } else {
                    use joinery::Joinable;
                    let target = data.remove(0);
                    let args = data.join_with(' ').to_string();
                    return read_query(terminal, executors, target, args);
                }
            }
            Action::Exit => return,
            Action::Complete => complete(),
            Action::Edit(action) => buffer.edit(&action),
            Action::MoveCursor(scope) => buffer.move_cursor(&scope),
        }

        terminal.clear_error();
    }
}

pub(super) fn run() {
    let terminal = terminal::new();

    let executors = match core::executors::load_default() {
        Ok(executors) => executors,
        Err(e) => terminal::fatal(e),
    };

    read_target(terminal, executors, None);
}
