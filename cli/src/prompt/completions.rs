pub(super) struct Completions {
    data: Vec<String>,
    selected: Option<usize>,
}

pub(super) fn new(data: Vec<String>) -> Completions {
    Completions {
        data,
        selected: None,
    }
}

impl Completions {
    pub(super) fn select_up(&mut self) -> Option<&str> {
        if self.data.is_empty() {
            None
        } else {
            self.selected = Some(self.selected.map_or_else(
                || self.data.len() - 1,
                |selected| {
                    if selected == 0 {
                        self.data.len() - 1
                    } else {
                        selected - 1
                    }
                },
            ));

            Some(&self.data[self.selected.unwrap()])
        }
    }

    pub(super) fn select_down(&mut self) -> Option<&str> {
        if self.data.is_empty() {
            None
        } else {
            self.selected = Some(self.selected.map_or(0, |selected| {
                if selected == self.data.len() - 1 {
                    0
                } else {
                    selected + 1
                }
            }));
            Some(&self.data[self.selected.unwrap()])
        }
    }

    pub(super) fn data(&self) -> &Vec<String> {
        &self.data
    }

    pub(super) fn selected(&self) -> &Option<usize> {
        &self.selected
    }
}

// impl<F> Completions<F>
//     where
//         F: Fn(&str) -> Vec<String>,
// {
//     pub(super) fn suggestion(&self) -> &String {
//         &self.suggestion
//     }
//
//     pub(super) fn generate(&mut self, buffer: &super::buffer::Buffer) {
//         if buffer.data_raw().is_empty() {
//             self.suggestion.clear();
//             return;
//         }
//
//         let data = buffer.data();
//
//         if let Some(suggestion) = (self.fetcher)(&data) {
//             self.suggestion = String::from(&suggestion[data.len()..suggestion.len()]);
//         } else {
//             self.suggestion.clear();
//         }
//     }
//
//     pub(super) fn take(&mut self) -> String {
//         let mut new = String::new();
//         std::mem::swap(&mut self.suggestion, &mut new);
//         new
//     }
//
//     pub(super) fn take_next_word(&mut self) -> String {
//         let suggestion = self.suggestion.chars().collect::<Vec<char>>();
//         let index = super::navigation::next_word(0, &suggestion);
//         self.suggestion = suggestion[index..suggestion.len()]
//             .iter()
//             .collect::<String>();
//         suggestion[0..index].iter().collect::<String>()
//     }
// }
