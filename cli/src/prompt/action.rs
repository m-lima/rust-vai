pub(super) enum Action {
    Edit(EditAction),
    MoveCursor(Scope),
    Execute,
    Cancel,
    Complete,
    Noop,
}

pub(super) enum EditAction {
    Write(char),
    Delete(Scope),
}

pub(super) enum Scope {
    Back,
    Forward,
    BackWord,
    ForwardWord,
    BackAll,
    ForwardAll,
    All,
    WordAll,
}

fn control(e: &crossterm::event::KeyEvent) -> bool {
    e.modifiers == crossterm::event::KeyModifiers::CONTROL
}

fn alt(e: &crossterm::event::KeyEvent) -> bool {
    e.modifiers == crossterm::event::KeyModifiers::ALT
}

pub(super) fn read() -> Action {
    match crossterm::event::read() {
        Ok(crossterm::event::Event::Key(e)) => match e.code {
            crossterm::event::KeyCode::Enter => Action::Execute,
            crossterm::event::KeyCode::Tab => Action::Complete,
            crossterm::event::KeyCode::Backspace => Action::Edit(EditAction::Delete(Scope::Back)),
            crossterm::event::KeyCode::Delete => Action::Edit(EditAction::Delete(Scope::Forward)),
            crossterm::event::KeyCode::Right => Action::MoveCursor(Scope::Forward),
            crossterm::event::KeyCode::Left => Action::MoveCursor(Scope::Back),
            crossterm::event::KeyCode::Home => Action::MoveCursor(Scope::BackAll),
            crossterm::event::KeyCode::End => Action::MoveCursor(Scope::ForwardAll),
            crossterm::event::KeyCode::Char(c) => {
                if control(&e) {
                    match c {
                        'm' => Action::Execute,
                        'c' => Action::Cancel,

                        'b' => Action::MoveCursor(Scope::Back),
                        'f' => Action::MoveCursor(Scope::Forward),
                        'a' => Action::MoveCursor(Scope::BackAll),
                        'e' => Action::MoveCursor(Scope::ForwardAll),

                        'j' => Action::Edit(EditAction::Delete(Scope::BackWord)),
                        'k' => Action::Edit(EditAction::Delete(Scope::ForwardWord)),
                        'h' => Action::Edit(EditAction::Delete(Scope::BackAll)),
                        'l' => Action::Edit(EditAction::Delete(Scope::ForwardAll)),
                        'w' => Action::Edit(EditAction::Delete(Scope::WordAll)),
                        'u' => Action::Edit(EditAction::Delete(Scope::All)),
                        _ => Action::Noop,
                    }
                } else if alt(&e) {
                    match c {
                        'b' => Action::MoveCursor(Scope::BackWord),
                        'f' => Action::MoveCursor(Scope::ForwardWord),
                        _ => Action::Noop,
                    }
                } else {
                    Action::Edit(EditAction::Write(c))
                }
            }
            _ => Action::Noop,
        },
        Ok(_) => Action::Noop,
        Err(e) => super::terminal::fatal(e),
    }
}
