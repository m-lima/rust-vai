pub(super) struct Suggester {
    data: String,
    suggestions: Vec<String>,
}

pub(super) fn new(suggestions: Vec<String>) -> Suggester {
    Suggester {
        data: String::new(),
        suggestions,
    }
}

impl Suggester {
    pub(super) fn data(&self) -> &String {
        &self.data
    }

    pub(super) fn generate(&mut self, buffer: &super::buffer::Buffer) {
        if buffer.data_raw().is_empty() {
            self.data.clear();
            return;
        }

        let data = buffer.data();

        if let Some(suggestion) = self
            .suggestions
            .iter()
            .find(|suggestion| suggestion.starts_with(&data))
            .map(|suggestion| String::from(&suggestion[data.len()..suggestion.len()]))
        {
            self.data = suggestion;
        } else {
            self.data.clear();
        }
    }

    pub(super) fn take(&mut self) -> String {
        let mut new = String::new();
        std::mem::swap(&mut self.data, &mut new);
        new
    }

    pub(super) fn take_next_word(&mut self) -> String {
        let suggestion = self.data.chars().collect::<Vec<char>>();
        let index = super::navigation::next_word(0, &suggestion);
        self.data = suggestion[index..suggestion.len()]
            .iter()
            .collect::<String>();
        suggestion[0..index].iter().collect::<String>()
    }
}
