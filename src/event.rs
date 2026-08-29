use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Refresh,
    NextProvider,
    PreviousProvider,
    ToggleCollapse,
    OpenUpdateModal,
    NextUpdateAction,
    PreviousUpdateAction,
    ConfirmUpdateAction,
    CloseUpdateModal,
    Quit,
    Ignore,
}

pub fn action_for_key(key: KeyEvent, update_modal_open: bool) -> Action {
    if key.kind == KeyEventKind::Release {
        return Action::Ignore;
    }

    if matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    if update_modal_open {
        return match (key.code, key.modifiers) {
            (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE) => Action::NextUpdateAction,
            (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE) => Action::PreviousUpdateAction,
            (KeyCode::Enter, KeyModifiers::NONE) => Action::ConfirmUpdateAction,
            (KeyCode::Esc | KeyCode::Char('q'), KeyModifiers::NONE) => Action::CloseUpdateModal,
            _ => Action::Ignore,
        };
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        (KeyCode::Char('u'), KeyModifiers::NONE) => Action::OpenUpdateModal,
        (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE) => Action::NextProvider,
        (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE) => Action::PreviousProvider,
        (KeyCode::Char(' ') | KeyCode::Enter, KeyModifiers::NONE) => Action::ToggleCollapse,
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
            action_for_key(press(KeyCode::Char('r'), KeyModifiers::NONE), false),
            Action::Refresh
        );
    }

    #[test]
    fn q_and_ctrl_c_quit() {
        assert_eq!(
            action_for_key(press(KeyCode::Char('q'), KeyModifiers::NONE), false),
            Action::Quit
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('c'), KeyModifiers::CONTROL), false,),
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

        assert_eq!(action_for_key(release, false), Action::Ignore);
    }

    #[test]
    fn navigation_and_collapse_keys_map_to_dashboard_actions() {
        for code in [KeyCode::Char('j'), KeyCode::Down] {
            assert_eq!(
                action_for_key(press(code, KeyModifiers::NONE), false),
                Action::NextProvider
            );
        }

        for code in [KeyCode::Char('k'), KeyCode::Up] {
            assert_eq!(
                action_for_key(press(code, KeyModifiers::NONE), false),
                Action::PreviousProvider
            );
        }

        for code in [KeyCode::Char(' '), KeyCode::Enter] {
            assert_eq!(
                action_for_key(press(code, KeyModifiers::NONE), false),
                Action::ToggleCollapse
            );
        }
    }

    #[test]
    fn u_opens_update_modal_from_dashboard() {
        assert_eq!(
            action_for_key(press(KeyCode::Char('u'), KeyModifiers::NONE), false),
            Action::OpenUpdateModal
        );
    }

    #[test]
    fn modal_keys_override_dashboard_actions() {
        assert_eq!(
            action_for_key(press(KeyCode::Down, KeyModifiers::NONE), true),
            Action::NextUpdateAction
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('k'), KeyModifiers::NONE), true),
            Action::PreviousUpdateAction
        );
        assert_eq!(
            action_for_key(press(KeyCode::Enter, KeyModifiers::NONE), true),
            Action::ConfirmUpdateAction
        );
        assert_eq!(
            action_for_key(press(KeyCode::Esc, KeyModifiers::NONE), true),
            Action::CloseUpdateModal
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('q'), KeyModifiers::NONE), true),
            Action::CloseUpdateModal
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('r'), KeyModifiers::NONE), true),
            Action::Ignore
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('c'), KeyModifiers::CONTROL), true),
            Action::Quit
        );
    }
}
