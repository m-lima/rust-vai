use std::io::Write;

static BASE_PROMPT_SIZE: u16 = 6;

pub(super) struct Terminal {
    has_error: bool,
    prompt_start: u16,
    cursor_position: u16,
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
            print_error_raw(message).flush();
            return;
        }
        if let Some(message) = payload.downcast_ref::<String>() {
            print_error_raw(message).flush();
            return;
        }
        print_error_raw("unhandled exception");
    }));

    Terminal {
        has_error: false,
        prompt_start: BASE_PROMPT_SIZE,
        cursor_position: BASE_PROMPT_SIZE,
    }
}

impl Terminal {
    pub(super) fn prompt_size(&self) -> u16 {
        self.prompt_start
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

    pub(super) fn prompt(&mut self, secondary_prompt: Option<&str>) {
        self.prompt_start = BASE_PROMPT_SIZE;

        let mut stdout = std::io::stdout();
        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            crossterm::style::SetForegroundColor(crossterm::style::Color::DarkGreen),
            crossterm::style::Print("vai"),
            crossterm::style::ResetColor,
        );

        if let Some(secondary_prompt) = secondary_prompt {
            let (effective_prompt, size) = Terminal::clip_prompt(secondary_prompt);

            self.prompt_start += size;

            crossterm::queue!(
                stdout,
                crossterm::style::Print("|"),
                crossterm::style::SetForegroundColor(crossterm::style::Color::Green),
                crossterm::style::Print(effective_prompt),
                crossterm::style::ResetColor,
            );
        }

        crossterm::queue!(stdout, crossterm::style::Print("> "),);
        stdout.flush();

        self.cursor_position = self.prompt_start;
    }

    pub(super) fn print(&mut self, buffer: &super::buffer::Buffer, suggestion: &str) {
        self.cursor_position = self.prompt_start + buffer.position();
        let mut width = usize::from(
            crossterm::terminal::size().map_or_else(|_| u16::max_value(), |size| size.0) + 1
                - self.prompt_start,
        );

        let mut stdout = std::io::stdout();
        crossterm::queue!(stdout, crossterm::cursor::MoveToColumn(self.prompt_start));

        // Main buffer
        {
            let data = buffer.data_raw();
            let char_len = data.len();

            if char_len < width {
                crossterm::queue!(stdout, crossterm::style::Print(buffer.data()));
                width -= char_len;
            } else {
                let position = usize::from(buffer.position());
                let data = if position > width {
                    data[position - width..position].iter().collect::<String>()
                } else {
                    data[0..width].iter().collect::<String>()
                };
                crossterm::queue!(stdout, crossterm::style::Print(data));
                width = 0;
            }
        }

        // Suggestion buffer
        {
            if width > 0 && !suggestion.is_empty() {
                let data = if suggestion.len() < width {
                    String::from(suggestion)
                } else {
                    suggestion.chars().take(width).collect::<String>()
                };

                crossterm::queue!(
                    stdout,
                    crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
                    crossterm::style::Print(data),
                    crossterm::style::ResetColor,
                );
            }
        }

        crossterm::queue!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            crossterm::cursor::MoveToColumn(self.cursor_position),
        );

        stdout.flush();
    }

    pub(super) fn clear_error(&mut self) {
        if self.has_error {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveDown(1),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                crossterm::cursor::MoveUp(1),
                crossterm::cursor::MoveToColumn(self.cursor_position),
            );
            self.has_error = false;
        }
    }

    pub(super) fn print_error<M: std::fmt::Display>(&mut self, message: M) {
        let mut stdout = print_error_raw(message);

        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveUp(1),
            crossterm::cursor::MoveToColumn(self.cursor_position),
        );
        stdout.flush();
        self.has_error = true;
    }
}

fn print_error_raw<M: std::fmt::Display>(message: M) -> std::io::Stdout {
    let mut stdout = std::io::stdout();

    crossterm::queue!(
        stdout,
        crossterm::style::Print("\n"),
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
