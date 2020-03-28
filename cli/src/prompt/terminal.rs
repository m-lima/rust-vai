use std::io::Write;

static BASE_PROMPT_SIZE: u16 = 6;

pub(super) struct Terminal {
    has_error: bool,
    completion_lines: u16,
    prompt_start: u16,
    cursor_position: u16,
    stdout: std::io::Stdout,
}

impl std::ops::Drop for Terminal {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode();
        crossterm::execute!(
            std::io::stdout(),
            crossterm::style::ResetColor,
            crossterm::style::Print('\n'),
        );
    }
}

pub(super) fn new() -> Terminal {
    crossterm::terminal::enable_raw_mode();

    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        if let Some(message) = payload.downcast_ref::<&str>() {
            print_error_internal(message).flush();
            return;
        }
        if let Some(message) = payload.downcast_ref::<String>() {
            print_error_internal(message).flush();
            return;
        }
        print_error_internal("unhandled exception").flush();
    }));

    Terminal {
        has_error: false,
        completion_lines: 0,
        prompt_start: BASE_PROMPT_SIZE,
        cursor_position: BASE_PROMPT_SIZE,
        stdout: std::io::stdout(),
    }
}

// Allowed because we cap at 32
#[allow(clippy::cast_possible_truncation)]
fn clip_prompt(prompt: &str) -> (&str, u16) {
    let effective_prompt = if prompt.len() > 32 {
        &prompt[0..32]
    } else {
        prompt
    };

    let size = effective_prompt.len() as u16 + 1;

    (effective_prompt, size)
}

impl Terminal {
    pub(super) fn set_prompt(&mut self, secondary_prompt: &Option<&str>) {
        self.prompt_start = BASE_PROMPT_SIZE;

        self.clear_completions();

        crossterm::queue!(
            self.stdout,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            crossterm::style::SetForegroundColor(crossterm::style::Color::DarkGreen),
            crossterm::style::Print("vai"),
            crossterm::style::ResetColor,
        );

        if let Some(secondary_prompt) = secondary_prompt {
            let (effective_prompt, size) = clip_prompt(secondary_prompt);

            self.prompt_start += size;

            crossterm::queue!(
                self.stdout,
                crossterm::style::Print("|"),
                crossterm::style::SetForegroundColor(crossterm::style::Color::Green),
                crossterm::style::Print(effective_prompt),
                crossterm::style::ResetColor,
            );
        }

        crossterm::queue!(self.stdout, crossterm::style::Print("> "),);

        self.cursor_position = self.prompt_start;
    }

    pub(super) fn print(&mut self, context: &super::Context) {
        let mut width = usize::from(
            crossterm::terminal::size().map_or_else(|_| u16::max_value(), |size| size.0) + 1
                - self.prompt_start,
        );
        let position = std::cmp::min(*context.buffer.position(), width);

        // Allowed because we are never larger than `width`
        #[allow(clippy::cast_possible_truncation)]
        {
            self.cursor_position = self.prompt_start + position as u16;
        }

        self.print_buffer(&context.buffer, &mut width, position);
        self.print_suggester(&context.suggester, width);
        self.print_completions(&context.completions);

        crossterm::queue!(
            self.stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            crossterm::cursor::MoveToColumn(self.cursor_position),
        );

        self.stdout.flush();
    }

    fn print_buffer(&mut self, buffer: &super::buffer::Buffer, width: &mut usize, position: usize) {
        crossterm::queue!(
            self.stdout,
            crossterm::cursor::MoveToColumn(self.prompt_start)
        );

        let data = buffer.data_raw();
        let char_len = data.len();

        if char_len < *width {
            crossterm::queue!(self.stdout, crossterm::style::Print(buffer.data()));
            *width -= char_len;
        } else {
            let data = if position > *width {
                data[position - *width..position].iter().collect::<String>()
            } else {
                data[0..*width].iter().collect::<String>()
            };
            crossterm::queue!(self.stdout, crossterm::style::Print(data));
            *width = 0;
        }
    }

    fn print_suggester(&mut self, suggester: &super::suggester::Suggester, width: usize) {
        let data = suggester.data();
        if width > 0 && !data.is_empty() {
            let data = if data.len() < width {
                String::from(data)
            } else {
                data.chars().take(width).collect::<String>()
            };

            crossterm::queue!(
                self.stdout,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
                crossterm::style::Print(data),
                crossterm::style::ResetColor,
            );
        }
    }

    fn clear_error(&mut self) {
        if self.has_error {
            crossterm::queue!(
                self.stdout,
                crossterm::cursor::MoveDown(1),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                crossterm::cursor::MoveUp(1),
                crossterm::cursor::MoveToColumn(self.cursor_position),
            );
            self.has_error = false;
        }
    }

    pub(super) fn print_error<M: std::fmt::Display>(&mut self, message: M) {
        self.clear_completions();
        print_error_internal(message);

        crossterm::queue!(
            self.stdout,
            crossterm::cursor::MoveUp(1),
            crossterm::cursor::MoveToColumn(self.cursor_position),
        );
        self.stdout.flush();
        self.has_error = true;
    }

    fn clear_completions(&mut self) {
        if self.completion_lines > 0 {
            for _ in 0..self.completion_lines {
                crossterm::queue!(
                    self.stdout,
                    crossterm::cursor::MoveDown(1),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                );
            }

            crossterm::queue!(
                self.stdout,
                crossterm::cursor::MoveUp(self.completion_lines),
                crossterm::cursor::MoveToColumn(self.cursor_position),
            );

            self.completion_lines = 0;
        }
    }

    fn print_completions(&mut self, completions: &Option<super::completions::Completions>) {
        self.clear_completions();
        if let Some(completions) = completions {
            self.clear_error();

            let selected = completions.selected().unwrap_or_else(usize::max_value);

            for completion in completions.data() {
                crossterm::queue!(
                    self.stdout,
                    crossterm::style::Print('\n'),
                    crossterm::cursor::MoveToColumn(0),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                );

                if usize::from(self.completion_lines) == selected {
                    crossterm::queue!(
                        self.stdout,
                        crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
                        crossterm::style::Print(completion),
                        crossterm::style::ResetColor,
                    );
                } else {
                    crossterm::queue!(self.stdout, crossterm::style::Print(completion));
                }

                self.completion_lines += 1;
            }

            crossterm::queue!(
                self.stdout,
                crossterm::cursor::MoveUp(self.completion_lines),
                crossterm::cursor::MoveToColumn(self.cursor_position),
            );
        }
    }
}

fn print_error_internal<M: std::fmt::Display>(message: M) -> std::io::Stdout {
    let mut stdout = std::io::stdout();
    crossterm::queue!(
        stdout,
        crossterm::style::Print('\n'),
        crossterm::cursor::MoveToColumn(0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
        crossterm::style::Print("Error: "),
        crossterm::style::ResetColor,
        crossterm::style::Print(message.to_string()),
    );
    stdout
}

pub(super) fn fatal<M: std::fmt::Display>(message: M) -> ! {
    panic!(message.to_string());
}
