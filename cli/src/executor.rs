use vai_core as core;

use crate::{Error, Result};

fn prompt(executor: Option<&core::executors::Executor>) -> impl rucline::prompt::Builder {
    use rucline::crossterm::style::Colorize;
    if let Some(executor) = executor {
        rucline::prompt::Prompt::from(format!(
            "{}::{}> ",
            "vai".green(),
            executor.name().dark_green()
        ))
    } else {
        rucline::prompt::Prompt::from(format!("{}> ", "vai".green()))
    }
}

fn move_cursor_to_executor(buffer: &mut rucline::Buffer) {
    // Cursor at zero never fails
    buffer.set_cursor(0).unwrap();
    buffer.move_cursor(
        rucline::actions::Range::Word,
        rucline::actions::Direction::Forward,
    );
    buffer.move_cursor(
        rucline::actions::Range::Single,
        rucline::actions::Direction::Backward,
    );
}

fn extract_trailing_buffer(buffer: &str) -> rucline::Buffer {
    buffer
        .find(' ')
        .map(|index| buffer[index..].trim_start().into())
        .unwrap_or_else(rucline::Buffer::new)
}

fn interactive_prompt(
    mut buffer: rucline::Buffer,
    executors: &core::executors::Executors,
) -> Result {
    use rucline::prompt::Builder;

    let targets = executors.list_targets();
    loop {
        move_cursor_to_executor(&mut buffer);

        if let rucline::Outcome::Accepted(accepted) = prompt(None)
            .buffer(buffer.clone())
            .completer_ref(&targets)
            .suggester_ref(&targets)
            .erase_after_read(true)
            .read_line()?
        {
            let trimmed = accepted.trim_start();
            if let Some(executor) = trimmed.split(' ').next().and_then(|t| executors.find(t)) {
                let history = executor.history().unwrap_or_else(|_| Vec::new());

                let susggestions = |b: &rucline::Buffer| -> Vec<std::borrow::Cow<'_, str>> {
                    executor
                        .suggest(b)
                        .unwrap_or_else(|_| Vec::new())
                        .into_iter()
                        .map(|s| s.into())
                        .collect()
                };

                match prompt(Some(executor))
                    .buffer(extract_trailing_buffer(trimmed))
                    .completer(history)
                    .suggester_fn(susggestions)
                    .erase_after_read(true)
                    .read_line()?
                {
                    rucline::Outcome::Accepted(accepted) => {
                        return executor.execute(&accepted).map_err(Error::Core);
                    }
                    rucline::Outcome::Canceled(canceled) => {
                        buffer = format!("{} {}", executor.name(), canceled).into();
                        continue;
                    }
                }
            } else {
                buffer = trimmed.into();
                continue;
            }
        } else {
            return Ok(());
        }
    }
}

pub(super) fn execute(args: Vec<String>) -> Result {
    let executors = core::executors::load_default()?;

    if !args.is_empty() {
        if let Some(executor) = executors.find(&args[0]) {
            return executor
                .execute(&args.into_iter().skip(1).collect::<Vec<_>>().join(" "))
                .map_err(Error::Core);
        }
    }

    interactive_prompt(args.join(" ").into(), &executors)
}
