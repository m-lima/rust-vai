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
