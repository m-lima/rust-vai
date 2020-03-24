use vai_core as core;

pub(super) struct Context {
    executors: core::executors::Executors,
    target: Option<String>,
    buffer: super::buffer::Buffer,
    suggester: super::suggester::Suggester,
    completer: Option<super::completions::Completions>,
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
        completer: None,
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

    pub(super) fn upgrade(&mut self) {
        self.completer = None;

        let mut old_buffer = self
            .buffer
            .data()
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>();

        if !old_buffer.is_empty() {
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
            };
        }
    }

    pub(super) fn downgrade(&mut self) {
        self.completer = None;

        if let Some(target) = &mut self.target {
            let suggester = target_suggester(&executors);
            
            // TODO move cursor to where `target` ends
            target.push_str(&self.buffer.data());

            self.target = None;
            self.buffer.set_str(&target);
            self.suggester = suggester;
        }
    }
}
