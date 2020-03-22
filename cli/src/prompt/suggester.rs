pub(super) struct Suggester<F>
where
    F: Fn(&str) -> Option<String>,
{
    suggestion: String,
    fetcher: F,
}

pub(super) fn new<F>(fetcher: F) -> Suggester<F>
where
    F: Fn(&str) -> Option<String>,
{
    Suggester {
        suggestion: String::new(),
        fetcher,
    }
}

impl<F> Suggester<F>
where
    F: Fn(&str) -> Option<String>,
{
    pub(super) fn suggestion(&self) -> &String {
        &self.suggestion
    }

    pub(super) fn generate(&mut self, buffer: &super::buffer::Buffer) {
        if buffer.data_raw().is_empty() {
            self.suggestion.clear();
            return;
        }

        let data = buffer.data();

        if let Some(suggestion) = (self.fetcher)(&data) {
            self.suggestion = String::from(&suggestion[data.len()..suggestion.len()]);
        } else {
            self.suggestion.clear();
        }
    }

    pub(super) fn take(&mut self) -> String {
        let mut new = String::new();
        std::mem::swap(&mut self.suggestion, &mut new);
        new
    }

    pub(super) fn take_next_word(&mut self) -> String {
        let suggestion = self.suggestion.chars().collect::<Vec<char>>();
        let index = super::navigation::next_word(0, &suggestion);
        self.suggestion = suggestion[index..suggestion.len()]
            .iter()
            .collect::<String>();
        suggestion[0..index].iter().collect::<String>()
    }
}
