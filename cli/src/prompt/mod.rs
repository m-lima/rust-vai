#![allow(unused_must_use)]

use vai_core as core;

mod action;
mod buffer;
mod completions;
// mod context;
mod navigation;
mod suggester;
mod terminal;

fn execute(target: String, buffer: &buffer::Buffer) {
    let mut args = vec![target];
    buffer
        .data()
        .split_whitespace()
        .for_each(|arg| args.push(String::from(arg)));

    if let Err(e) = super::execute(args) {
        terminal::fatal(e);
    }
}

fn edit<F>(
    action: &action::EditAction,
    buffer: &mut buffer::Buffer,
    suggester: &mut suggester::Suggester<F>,
) where
    F: Fn(&str) -> Option<String>,
{
    buffer.edit(&action);
    suggester.generate(&buffer);
}

fn move_cursor<F>(
    scope: &action::Scope,
    buffer: &mut buffer::Buffer,
    suggester: &mut suggester::Suggester<F>,
) where
    F: Fn(&str) -> Option<String>,
{
    if buffer.at_end() {
        match scope {
            action::Scope::Forward | action::Scope::ForwardAll => {
                buffer.write_str(&suggester.take());
            }
            action::Scope::ForwardWord => {
                buffer.write_str(&suggester.take_next_word());
            }
            _ => {}
        }
    }
    buffer.move_cursor(&scope)
}

fn read_query(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    target: String,
    args: Option<String>,
) {
    let executor = if let Some(executor) = executors.find(&target) {
        executor
    } else {
        terminal.print_error("Invalid target");
        let mut buffer = target;
        if let Some(args) = args {
            buffer.push(' ');
            buffer.push_str(&args);
        }
        return read_target(terminal, executors, Some(buffer));
    };

    terminal.prompt(Some(&target));
    let mut buffer = buffer::new(u16::max_value() - terminal.prompt_size());
    let mut suggester = suggester::new(|query| {
        executor
            .strict_history(query, 1)
            .ok()
            .and_then(|history| history.first().map(String::from))
    });
    let mut completions: Option<completions::Completions> = None;

    if let Some(args) = args {
        buffer.write_str(&args);
        suggester.generate(&buffer);
    }

    loop {
        use action::Action;

        terminal.print(&buffer, suggester.suggestion());

        match action::read() {
            Action::Noop => continue,
            Action::Exit => return,
            Action::Execute => return execute(target, &buffer),
            Action::Cancel => return read_target(terminal, executors, Some(target)),
            Action::Complete(direction) => {
                if let Some(completions) = completions.as_mut() {
                    use action::Direction;
                    if let Some(completion) = match direction {
                        Direction::Down => completions.select_down(),
                        Direction::Up => completions.select_up(),
                    } {
                        buffer.set_str(completion);
                        suggester.generate(&buffer);
                    }
                } else {
                    completions = Some(completions::new(
                            {
                                let query = buffer.data();
                                let mut completions = executor
                                    .fuzzy_history(&query, 10)
                                    .unwrap_or_else(|_| vec![]);
                                completions.extend(executor.suggest(&query).unwrap_or_else(|_| vec![]));
                                completions
                            },
                    ));
                }
                terminal.print_completions(completions.as_ref().unwrap());
            }
            Action::Edit(action) => {
                edit(&action, &mut buffer, &mut suggester);
                completions = None;
            },
            Action::MoveCursor(scope) => move_cursor(&scope, &mut buffer, &mut suggester),
        }

        terminal.clear_error();
    }
}

fn read_target(
    mut terminal: terminal::Terminal,
    executors: core::executors::Executors,
    args: Option<String>,
) {
    terminal.prompt(None);
    let mut buffer = buffer::new(u16::max_value() - terminal.prompt_size());
    // Allowed because I disagree with clippy's argument for readability
    #[allow(clippy::find_map)]
    let mut suggester = suggester::new(|target| {
        executors
            .list_targets()
            .iter()
            .find(|suggestion| suggestion.starts_with(target))
            .map(|suggestion| String::from(*suggestion))
    });
    let mut completions: Option<completions::Completions> = None;

    if let Some(args) = args {
        buffer.write_str(&args);
        suggester.generate(&buffer);
    }

    loop {
        use action::Action;

        terminal.print(&buffer, suggester.suggestion());

        match action::read() {
            Action::Noop | Action::Cancel => continue,
            Action::Execute => {
                let mut data = buffer
                    .data()
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>();

                if !data.is_empty() {
                    use joinery::Joinable;
                    let target = data.remove(0);
                    let args = if data.is_empty() {
                        None
                    } else {
                        Some(data.join_with(' ').to_string())
                    };
                    return read_query(terminal, executors, target, args);
                }

                return;
            }
            Action::Exit => return,
            Action::Complete(direction) => {
                if let Some(completions) = completions.as_mut() {
                    use action::Direction;
                    if let Some(completion) = match direction {
                        Direction::Down => completions.select_down(),
                        Direction::Up => completions.select_up(),
                    } {
                        buffer.set_str(completion);
                        suggester.generate(&buffer);
                    }
                } else {
                    completions = Some(completions::new(
                        executors
                            .list_targets()
                            .iter()
                            .map(|target| String::from(*target))
                            .collect(),
                    ));
                }
                terminal.print_completions(completions.as_ref().unwrap());
            }
            Action::Edit(action) => {
                edit(&action, &mut buffer, &mut suggester);
                completions = None;
                terminal.clear_completions();
            }
            Action::MoveCursor(scope) => move_cursor(&scope, &mut buffer, &mut suggester),
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
