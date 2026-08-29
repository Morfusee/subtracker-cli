use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Refresh,
    Quit,
    Ignore,
}

pub fn action_for_key(key: KeyEvent) -> Action {
    if key.kind == KeyEventKind::Release {
        return Action::Ignore;
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,
        (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            Action::Quit
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        _ => Action::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new_with_kind(code, modifiers, KeyEventKind::Press)
    }

    #[test]
    fn r_requests_refresh() {
        assert_eq!(
            action_for_key(press(KeyCode::Char('r'), KeyModifiers::NONE)),
            Action::Refresh
        );
    }

    #[test]
    fn q_and_ctrl_c_quit() {
        assert_eq!(
            action_for_key(press(KeyCode::Char('q'), KeyModifiers::NONE)),
            Action::Quit
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('c'), KeyModifiers::CONTROL,)),
            Action::Quit
        );
    }

    #[test]
    fn key_release_is_ignored() {
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );

        assert_eq!(action_for_key(release), Action::Ignore);
    }
}
