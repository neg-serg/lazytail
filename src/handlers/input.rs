use crate::app::AppEvent;
use crate::app::{App, InputMode};
use crate::keymap::{Action, BindMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input and return corresponding events
/// Does not mutate app state directly - returns events to be processed
pub fn handle_input_event(key: KeyEvent, app: &App) -> Vec<AppEvent> {
    // If help is showing, handle help navigation or hide
    if app.help_scroll_offset.is_some() {
        return handle_help_mode(key);
    }

    // Dismiss warning popup on any key
    if app.warning_popup.is_some() {
        return vec![AppEvent::DismissWarning];
    }

    match app.input.mode {
        InputMode::EnteringFilter => handle_filter_input_mode(key),
        InputMode::EnteringLineJump => handle_line_jump_input_mode(key),
        InputMode::ZPending => handle_z_pending_mode(key, app),
        InputMode::SourcePanel => handle_source_panel_mode(key, app),
        InputMode::ConfirmClose => handle_confirm_close_mode(key),
        InputMode::Normal => handle_normal_mode(key, app),
    }
}

/// Handle keyboard input when help overlay is showing
fn handle_help_mode(key: KeyEvent) -> Vec<AppEvent> {
    match key.code {
        KeyCode::Char('q') => vec![AppEvent::Quit],
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::Quit]
        }
        KeyCode::Char('j') | KeyCode::Down => vec![AppEvent::ScrollHelpDown],
        KeyCode::Char('k') | KeyCode::Up => vec![AppEvent::ScrollHelpUp],
        // Any other key hides help
        _ => vec![AppEvent::HideHelp],
    }
}

/// Handle keyboard input in filter input mode
fn handle_filter_input_mode(key: KeyEvent) -> Vec<AppEvent> {
    match key.code {
        // Alt+C toggles case sensitivity (Ctrl+I doesn't work - same as Tab in terminals)
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => {
            vec![AppEvent::ToggleCaseSensitivity]
        }
        // Ctrl+A goes to start of line
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::CursorHome]
        }
        // Ctrl+E goes to end of line
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::CursorEnd]
        }
        KeyCode::Char(c) => vec![AppEvent::FilterInputChar(c)],
        KeyCode::Backspace => vec![AppEvent::FilterInputBackspace],
        KeyCode::Enter => vec![AppEvent::FilterInputSubmit],
        KeyCode::Esc => vec![AppEvent::FilterInputCancel, AppEvent::ClearFilter],
        KeyCode::Up => vec![AppEvent::HistoryUp],
        KeyCode::Down => vec![AppEvent::HistoryDown],
        // Tab toggles between Plain and Regex mode
        KeyCode::Tab => vec![AppEvent::ToggleFilterMode],
        // Cursor navigation
        KeyCode::Left => vec![AppEvent::CursorLeft],
        KeyCode::Right => vec![AppEvent::CursorRight],
        KeyCode::Home => vec![AppEvent::CursorHome],
        KeyCode::End => vec![AppEvent::CursorEnd],
        _ => vec![],
    }
}

/// Handle keyboard input in line jump input mode
fn handle_line_jump_input_mode(key: KeyEvent) -> Vec<AppEvent> {
    match key.code {
        // Ctrl+A goes to start of line
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::CursorHome]
        }
        // Ctrl+E goes to end of line
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::CursorEnd]
        }
        KeyCode::Char(c) => vec![AppEvent::LineJumpInputChar(c)],
        KeyCode::Backspace => vec![AppEvent::LineJumpInputBackspace],
        KeyCode::Enter => vec![AppEvent::LineJumpInputSubmit],
        KeyCode::Esc => vec![AppEvent::LineJumpInputCancel],
        // Cursor navigation
        KeyCode::Left => vec![AppEvent::CursorLeft],
        KeyCode::Right => vec![AppEvent::CursorRight],
        KeyCode::Home => vec![AppEvent::CursorHome],
        KeyCode::End => vec![AppEvent::CursorEnd],
        _ => vec![],
    }
}

/// Handle keyboard input in z pending mode (waiting for zz, zt, zb)
fn handle_z_pending_mode(key: KeyEvent, app: &App) -> Vec<AppEvent> {
    match app
        .keymap
        .resolve(BindMode::ZPending, key.code, key.modifiers)
    {
        Some(action) => action_events(action, app),
        // Any other key cancels z mode
        None => vec![AppEvent::ExitZMode],
    }
}

/// Handle keyboard input in source panel focus mode
fn handle_source_panel_mode(key: KeyEvent, app: &App) -> Vec<AppEvent> {
    match app.keymap.resolve(BindMode::Panel, key.code, key.modifiers) {
        Some(action) => action_events(action, app),
        None => vec![],
    }
}

/// Handle keyboard input in close confirmation mode
fn handle_confirm_close_mode(key: KeyEvent) -> Vec<AppEvent> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => vec![AppEvent::ConfirmCloseTab],
        KeyCode::Char('n') | KeyCode::Esc => vec![AppEvent::CancelCloseTab],
        KeyCode::Char('q') => vec![AppEvent::Quit],
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::Quit]
        }
        _ => vec![],
    }
}

/// Handle keyboard input in aggregation view mode
fn handle_aggregation_mode(key: KeyEvent) -> Vec<AppEvent> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => vec![AppEvent::AggregationDown],
        KeyCode::Char('k') | KeyCode::Up => vec![AppEvent::AggregationUp],
        KeyCode::Enter => vec![AppEvent::AggregationDrillDown],
        KeyCode::Esc => vec![AppEvent::AggregationBack],
        KeyCode::Char('g') => vec![AppEvent::AggregationJumpToStart],
        KeyCode::Char('G') => vec![AppEvent::AggregationJumpToEnd],
        KeyCode::Char('/') => vec![AppEvent::StartFilterInput],
        KeyCode::Char('q') => vec![AppEvent::Quit],
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            vec![AppEvent::Quit]
        }
        KeyCode::Char('?') => vec![AppEvent::ShowHelp],
        KeyCode::Tab => vec![AppEvent::FocusSourcePanel],
        KeyCode::Char(c @ '1'..='9') => {
            let index = (c as usize) - ('1' as usize);
            vec![AppEvent::SelectTab(index)]
        }
        _ => vec![],
    }
}

fn handle_normal_mode(key: KeyEvent, app: &App) -> Vec<AppEvent> {
    // In aggregation mode, use aggregation-specific keybindings
    if app.active_tab().source.mode == crate::app::ViewMode::Aggregation {
        return handle_aggregation_mode(key);
    }

    match app
        .keymap
        .resolve(BindMode::Normal, key.code, key.modifiers)
    {
        Some(action) => action_events(action, app),
        None => vec![],
    }
}

/// Translate a bound action into the application events it triggers.
///
/// This is the single place where chord semantics live; `input.rs` no longer
/// matches on `KeyCode` for the rebindable modes.
fn action_events(action: Action, app: &App) -> Vec<AppEvent> {
    match action {
        Action::Quit => vec![AppEvent::Quit],
        Action::ScrollDown => vec![AppEvent::ScrollDown, AppEvent::DisableFollowMode],
        Action::ScrollUp => vec![AppEvent::ScrollUp, AppEvent::DisableFollowMode],
        Action::ViewportDown => vec![AppEvent::ViewportDown, AppEvent::DisableFollowMode],
        Action::ViewportUp => vec![AppEvent::ViewportUp, AppEvent::DisableFollowMode],
        // The page size is appended by `collect_input_events` (needs the
        // terminal height), exactly as before.
        Action::PageDown => vec![AppEvent::DisableFollowMode],
        Action::PageUp => vec![AppEvent::DisableFollowMode],
        Action::HalfPageDown => vec![AppEvent::DisableFollowMode],
        Action::HalfPageUp => vec![AppEvent::DisableFollowMode],
        Action::JumpToStart => vec![AppEvent::JumpToStart, AppEvent::DisableFollowMode],
        Action::JumpToEnd => vec![AppEvent::JumpToEnd, AppEvent::DisableFollowMode],
        Action::ToggleFollow => vec![AppEvent::ToggleFollowMode],
        Action::StartFilter => vec![AppEvent::StartFilterInput],
        Action::StartLineJump => vec![AppEvent::StartLineJumpInput],
        Action::ShowHelp => vec![AppEvent::ShowHelp],
        Action::EnterZMode => vec![AppEvent::EnterZMode],
        Action::ToggleLineExpansion => vec![AppEvent::ToggleLineExpansion],
        Action::CollapseAll => vec![AppEvent::CollapseAll],
        Action::ToggleRaw => vec![AppEvent::ToggleRawMode],
        Action::ToggleTimestamps => vec![AppEvent::ToggleTimestamps],
        Action::ToggleWrap => vec![AppEvent::ToggleLineWrap],
        Action::CopyLine => vec![AppEvent::CopySelectedLine],
        Action::CloseTab => vec![AppEvent::CloseCurrentTab],
        Action::RefreshCombined => {
            if app.active_tab().is_combined {
                vec![AppEvent::RefreshCombinedView]
            } else {
                vec![]
            }
        }
        Action::ClearFilter => vec![AppEvent::ClearFilter],
        Action::FocusSourcePanel => vec![AppEvent::FocusSourcePanel],
        // Config names tabs 1-based (`tab_1`); AppEvent wants a 0-based index.
        Action::SelectTab(index) => vec![AppEvent::SelectTab(index.saturating_sub(1) as usize)],
        Action::ZCenter => vec![AppEvent::CenterView, AppEvent::ExitZMode],
        Action::ZTop => vec![AppEvent::ViewToTop, AppEvent::ExitZMode],
        Action::ZBottom => vec![AppEvent::ViewToBottom, AppEvent::ExitZMode],
        Action::ZCancel => vec![AppEvent::ExitZMode],
        Action::PanelUp => vec![AppEvent::SourcePanelUp],
        Action::PanelDown => vec![AppEvent::SourcePanelDown],
        Action::PanelToggleExpand => vec![AppEvent::ToggleCategoryExpand],
        Action::PanelSelect => vec![AppEvent::SelectSource],
        Action::PanelCloseTab => vec![AppEvent::CloseSelectedTab],
        Action::PanelCopyPath => vec![AppEvent::CopySourcePath],
        Action::PanelUnfocus => vec![AppEvent::UnfocusSourcePanel],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_app() -> (App, NamedTempFile) {
        let mut file = NamedTempFile::new().unwrap();
        for i in 0..10 {
            writeln!(file, "line{}", i).unwrap();
        }
        file.flush().unwrap();
        let app = App::new(vec![file.path().to_path_buf()], false).unwrap();
        (app, file)
    }

    #[test]
    fn test_follow_is_bound_to_f_and_capital_f() {
        let (app, _file) = create_test_app();
        for key in ['f', 'F'] {
            let events =
                handle_input_event(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE), &app);
            assert_eq!(events, vec![AppEvent::ToggleFollowMode], "key {key}");
        }
    }

    #[test]
    fn test_keymap_override_moves_follow_to_capital_f_only() {
        let (mut app, _file) = create_test_app();
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("follow".to_string(), vec!["F".to_string()]);
        app.keymap = crate::keymap::Keymap::build(&overrides).unwrap();

        assert_eq!(
            handle_input_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE), &app),
            vec![]
        );
        assert_eq!(
            handle_input_event(KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE), &app),
            vec![AppEvent::ToggleFollowMode]
        );
    }

    #[test]
    fn test_keymap_empty_list_unbinds_close_tab() {
        let (mut app, _file) = create_test_app();
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("close_tab".to_string(), Vec::new());
        app.keymap = crate::keymap::Keymap::build(&overrides).unwrap();

        assert_eq!(
            handle_input_event(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE), &app),
            vec![]
        );
    }

    #[test]
    fn test_keymap_page_down_via_ctrl_f() {
        let (app, _file) = create_test_app();
        let events = handle_input_event(
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            &app,
        );
        // Page size is appended by the event loop, which knows the terminal size.
        assert_eq!(events, vec![AppEvent::DisableFollowMode]);
    }

    #[test]
    fn test_quit_on_q() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::Quit]);
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::Quit]);
    }

    #[test]
    fn test_scroll_down() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::ScrollDown, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_scroll_up() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::ScrollUp, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_viewport_down_ctrl_e() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::ViewportDown, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_viewport_up_ctrl_y() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::ViewportUp, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_toggle_follow_mode() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ToggleFollowMode]);
    }

    #[test]
    fn test_start_filter_input() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::StartFilterInput]);
    }

    #[test]
    fn test_filter_input_char() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::FilterInputChar('a')]);
    }

    #[test]
    fn test_filter_input_backspace() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::FilterInputBackspace]);
    }

    #[test]
    fn test_filter_input_submit() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::FilterInputSubmit]);
    }

    #[test]
    fn test_filter_input_cancel() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::FilterInputCancel, AppEvent::ClearFilter]
        );
    }

    #[test]
    fn test_jump_to_start() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::JumpToStart, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_jump_to_end() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::JumpToEnd, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_show_help() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ShowHelp]);
    }

    #[test]
    fn test_hide_help_on_any_key() {
        let (mut app, _file) = create_test_app();
        app.help_scroll_offset = Some(0);

        // j/k scroll, other keys hide
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::HideHelp]);
    }

    #[test]
    fn test_scroll_help() {
        let (mut app, _file) = create_test_app();
        app.help_scroll_offset = Some(0);

        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ScrollHelpDown]);

        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ScrollHelpUp]);
    }

    #[test]
    fn test_quit_from_help_mode() {
        let (mut app, _file) = create_test_app();
        app.help_scroll_offset = Some(0);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::Quit]);
    }

    #[test]
    fn test_start_line_jump_input() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::StartLineJumpInput]);
    }

    #[test]
    fn test_line_jump_input_char() {
        let (mut app, _file) = create_test_app();
        app.start_line_jump_input();
        let key = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::LineJumpInputChar('5')]);
    }

    #[test]
    fn test_line_jump_input_backspace() {
        let (mut app, _file) = create_test_app();
        app.start_line_jump_input();
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::LineJumpInputBackspace]);
    }

    #[test]
    fn test_line_jump_input_submit() {
        let (mut app, _file) = create_test_app();
        app.start_line_jump_input();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::LineJumpInputSubmit]);
    }

    #[test]
    fn test_line_jump_input_cancel() {
        let (mut app, _file) = create_test_app();
        app.start_line_jump_input();
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::LineJumpInputCancel]);
    }

    #[test]
    fn test_filter_input_history_up() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::HistoryUp]);
    }

    #[test]
    fn test_filter_input_history_down() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::HistoryDown]);
    }

    #[test]
    fn test_tab_focuses_source_panel() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::FocusSourcePanel]);
    }

    #[test]
    fn test_select_tab_by_number() {
        let (app, _file) = create_test_app();

        // Test keys 1-9
        for i in 1..=9 {
            let key = KeyEvent::new(
                KeyCode::Char(char::from_digit(i, 10).unwrap()),
                KeyModifiers::NONE,
            );
            let events = handle_input_event(key, &app);
            assert_eq!(events, vec![AppEvent::SelectTab((i - 1) as usize)]);
        }
    }

    #[test]
    fn test_filter_input_cursor_left() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorLeft]);
    }

    #[test]
    fn test_filter_input_cursor_right() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorRight]);
    }

    #[test]
    fn test_filter_input_cursor_home() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorHome]);
    }

    #[test]
    fn test_filter_input_cursor_end() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorEnd]);
    }

    #[test]
    fn test_filter_input_ctrl_a_home() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorHome]);
    }

    #[test]
    fn test_filter_input_ctrl_e_end() {
        let (mut app, _file) = create_test_app();
        app.start_filter_input();
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorEnd]);
    }

    #[test]
    fn test_line_jump_input_cursor_left() {
        let (mut app, _file) = create_test_app();
        app.start_line_jump_input();
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorLeft]);
    }

    #[test]
    fn test_line_jump_input_cursor_right() {
        let (mut app, _file) = create_test_app();
        app.start_line_jump_input();
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CursorRight]);
    }

    #[test]
    fn test_confirm_close_y_confirms() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::ConfirmClose;
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ConfirmCloseTab]);
    }

    #[test]
    fn test_confirm_close_enter_confirms() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::ConfirmClose;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ConfirmCloseTab]);
    }

    #[test]
    fn test_confirm_close_n_cancels() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::ConfirmClose;
        let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CancelCloseTab]);
    }

    #[test]
    fn test_confirm_close_esc_cancels() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::ConfirmClose;
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CancelCloseTab]);
    }

    #[test]
    fn test_confirm_close_ignores_other_keys() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::ConfirmClose;
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert!(events.is_empty());
    }

    #[test]
    fn test_toggle_raw_mode() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ToggleRawMode]);
    }

    #[test]
    fn test_toggle_line_wrap() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::ToggleLineWrap]);
    }

    #[test]
    fn test_ctrl_w_closes_tab_not_wrap() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CloseCurrentTab]);
    }

    #[test]
    fn test_y_copies_selected_line() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CopySelectedLine]);
    }

    #[test]
    fn test_ctrl_y_viewport_up_not_copy() {
        let (app, _file) = create_test_app();
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        let events = handle_input_event(key, &app);
        assert_eq!(
            events,
            vec![AppEvent::ViewportUp, AppEvent::DisableFollowMode]
        );
    }

    #[test]
    fn test_source_panel_y_copies_path_not_line() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::SourcePanel;
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CopySourcePath]);
    }

    #[test]
    fn test_source_panel_x_closes_selected() {
        let (mut app, _file) = create_test_app();
        app.input.mode = InputMode::SourcePanel;
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let events = handle_input_event(key, &app);
        assert_eq!(events, vec![AppEvent::CloseSelectedTab]);
    }
}
