use crate::app::App;
use crate::keymap::{Action, BindMode, Keymap};
use crate::source::SourceStatus;
use crate::theme::UiColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

// Help overlay dimensions (as percentage of screen)
const HELP_POPUP_WIDTH_PERCENT: f32 = 0.6;
const HELP_POPUP_HEIGHT_PERCENT: f32 = 0.8;

/// Render the chords bound to an action as `a / b / c`, or `—` when unbound.
fn chords(keymap: &Keymap, mode: BindMode, action: Action) -> String {
    let list = keymap.chords_for(mode, action);
    if list.is_empty() {
        "—".to_string()
    } else {
        list.join(" / ")
    }
}

/// One help row: chords in a fixed column, then the description.
fn row(keymap: &Keymap, mode: BindMode, action: Action, description: &str) -> Line<'static> {
    Line::from(format!(
        "  {:<16}{}",
        chords(keymap, mode, action),
        description
    ))
}

/// The `1`-`9` tab shortcuts stay a range: every tab action shares the row.
fn line_tabs(_keymap: &Keymap) -> Line<'static> {
    Line::from("  1-9             Jump to tab")
}

/// One help row for several actions that share a description.
fn row_multi(
    keymap: &Keymap,
    mode: BindMode,
    actions: &[Action],
    description: &str,
) -> Line<'static> {
    let joined = actions
        .iter()
        .map(|action| chords(keymap, mode, *action))
        .collect::<Vec<_>>()
        .join(" / ");
    Line::from(format!("  {joined:<16}{description}"))
}

pub(super) fn render_help_overlay(
    f: &mut Frame,
    area: Rect,
    scroll_offset: usize,
    ui: &UiColors,
    keymap: &Keymap,
) {
    // Calculate centered popup area
    let popup_width = (area.width as f32 * HELP_POPUP_WIDTH_PERCENT) as u16;
    let popup_height = (area.height as f32 * HELP_POPUP_HEIGHT_PERCENT) as u16;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Help content. Chords come from the live keymap, so a rebind is
    // reflected here immediately; everything that is not rebindable
    // (input-mode keys, mouse) stays static.
    let help_lines = vec![
        Line::from(vec![Span::styled(
            "LazyTail - Quick Reference",
            Style::default().fg(ui.primary).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        row(
            keymap,
            BindMode::Normal,
            Action::ScrollDown,
            "Move selection down",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ScrollUp,
            "Move selection up",
        ),
        row_multi(
            keymap,
            BindMode::Normal,
            &[Action::JumpToStart, Action::JumpToEnd],
            "Jump to start / end",
        ),
        row_multi(
            keymap,
            BindMode::Normal,
            &[Action::PageDown, Action::PageUp],
            "Scroll by page",
        ),
        row_multi(
            keymap,
            BindMode::Normal,
            &[Action::ViewportDown, Action::ViewportUp],
            "Scroll viewport (vim-style)",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::StartLineJump,
            "Jump to line number",
        ),
        Line::from("  zz/zt/zb        Center/top/bottom view"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Filtering",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        row(
            keymap,
            BindMode::Normal,
            Action::StartFilter,
            "Start filter (live preview)",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ClearFilter,
            "Clear filter",
        ),
        Line::from("  Tab             Cycle Plain → Regex → Query"),
        Line::from("  Alt+C           Toggle case sensitivity"),
        Line::from("  ↑/↓             Browse filter history"),
        Line::from("  Enter           Apply filter"),
        Line::from("  Query mode      json | ... / logfmt | ..."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tabs",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        line_tabs(keymap),
        row(keymap, BindMode::Normal, Action::CloseTab, "Close tab"),
        row(
            keymap,
            BindMode::Normal,
            Action::FocusSourcePanel,
            "Toggle panel focus",
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Source Panel",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        row_multi(
            keymap,
            BindMode::Panel,
            &[Action::PanelDown, Action::PanelUp],
            "Navigate tree",
        ),
        row(
            keymap,
            BindMode::Panel,
            Action::PanelToggleExpand,
            "Expand/collapse category",
        ),
        row(
            keymap,
            BindMode::Panel,
            Action::PanelSelect,
            "Select source",
        ),
        row(
            keymap,
            BindMode::Panel,
            Action::PanelCloseTab,
            "Close selected source",
        ),
        row(
            keymap,
            BindMode::Panel,
            Action::PanelCopyPath,
            "Copy source path",
        ),
        row(
            keymap,
            BindMode::Panel,
            Action::PanelUnfocus,
            "Return to log view",
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "View",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        row(
            keymap,
            BindMode::Normal,
            Action::ToggleLineExpansion,
            "Expand/collapse line",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::CollapseAll,
            "Collapse all",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ToggleFollow,
            "Toggle follow mode",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ToggleRaw,
            "Toggle raw mode",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ToggleWrap,
            "Toggle line wrap",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ToggleTimestamps,
            "Toggle timestamps",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::CopyLine,
            "Copy line to clipboard",
        ),
        row(
            keymap,
            BindMode::Normal,
            Action::ShowHelp,
            "Toggle this help",
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Mouse",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Click source     Switch to tab"),
        Line::from("  Click log line   Select line"),
        Line::from("  Scroll wheel     Scroll log view"),
        Line::from("  Click category   Expand/collapse"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Side Panel Indicators",
            Style::default().fg(ui.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("F", Style::default().fg(ui.positive)),
            Span::raw("  Follow mode    "),
            Span::styled("*", Style::default().fg(ui.accent)),
            Span::raw("  Filter active"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("●", Style::default().fg(ui.positive)),
            Span::raw("  Source active  "),
            Span::styled("○", Style::default().fg(ui.muted)),
            Span::raw("  Source ended"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("⟳", Style::default().fg(ui.highlight)),
            Span::raw("  Loading"),
        ]),
        Line::from(""),
        row(keymap, BindMode::Normal, Action::Quit, "Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "j/k to scroll, any other key to close",
            Style::default().fg(ui.muted).add_modifier(Modifier::ITALIC),
        )]),
    ];

    let total_lines = help_lines.len();
    // Inner height = popup height - 2 (top/bottom border)
    let inner_height = popup_height.saturating_sub(2) as usize;
    // Clamp scroll offset so we don't scroll past the content
    let max_scroll = total_lines.saturating_sub(inner_height);
    let scroll = scroll_offset.min(max_scroll);

    let has_more_above = scroll > 0;
    let has_more_below = scroll < max_scroll;

    // Build title with scroll indicators
    let title = match (has_more_above, has_more_below) {
        (true, true) => " Help ↑↓ ",
        (true, false) => " Help ↑ ",
        (false, true) => " Help ↓ ",
        (false, false) => " Help ",
    };

    let help_paragraph = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().bg(ui.popup_bg)),
        )
        .style(Style::default().bg(ui.popup_bg).fg(ui.fg))
        .scroll((scroll as u16, 0));

    // Clear the area first to remove background content
    f.render_widget(Clear, popup_area);
    f.render_widget(help_paragraph, popup_area);
}

pub(super) fn render_confirm_close_dialog(f: &mut Frame, area: Rect, app: &App) {
    let ui = &app.theme.ui;
    let tab_index = match &app.pending_close_tab {
        Some((idx, name))
            if *idx < app.tab_mgr.tabs.len() && app.tab_mgr.tabs[*idx].source.name == *name =>
        {
            *idx
        }
        _ => return,
    };

    let tab = &app.tab_mgr.tabs[tab_index];
    let tab_name = &tab.source.name;
    let is_last = app.tab_mgr.tabs.len() <= 1;
    let will_delete =
        tab.source.source_status == Some(SourceStatus::Ended) && tab.source.source_path.is_some();

    // Truncate name to fit in popup
    let max_name_len: usize = 30;
    let display_name = if tab_name.len() > max_name_len {
        let truncate_at = max_name_len.saturating_sub(3);
        let boundary = tab_name.floor_char_boundary(truncate_at);
        format!("{}...", &tab_name[..boundary])
    } else {
        tab_name.clone()
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Close "),
            Span::styled(
                &display_name,
                Style::default().fg(ui.primary).add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
    ];

    // Add context note
    if will_delete {
        lines.push(Line::from(vec![Span::styled(
            "  Source file will be deleted",
            Style::default().fg(ui.negative),
        )]));
    } else if is_last {
        lines.push(Line::from(vec![Span::styled(
            "  This will quit the application",
            Style::default().fg(ui.muted),
        )]));
    } else {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("y/Enter", Style::default().fg(ui.positive)),
        Span::raw(" confirm  "),
        Span::styled("n/Esc", Style::default().fg(ui.negative)),
        Span::raw(" cancel"),
    ]));

    let popup_width = 44.min(area.width.saturating_sub(4));
    let popup_height = 6;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Close Source ")
                .style(Style::default().bg(ui.popup_bg)),
        )
        .style(Style::default().bg(ui.popup_bg).fg(ui.fg));

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

pub(super) fn render_warning_popup(f: &mut Frame, area: Rect, message: &str, ui: &UiColors) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}", message),
            Style::default()
                .fg(ui.severity_error)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Press "),
            Span::styled("any key", Style::default().fg(ui.muted)),
            Span::raw(" to dismiss"),
        ]),
    ];

    let popup_width = 52.min(area.width.saturating_sub(4));
    let popup_height = 6;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Warning ")
                .style(Style::default().bg(ui.popup_bg)),
        )
        .style(Style::default().bg(ui.popup_bg).fg(ui.fg));

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}
