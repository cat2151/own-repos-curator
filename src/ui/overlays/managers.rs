use crate::app::{GroupManagerState, TagManagerState};
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, Wrap},
};

pub(in crate::ui) fn render_tag_manager(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &TagManagerState,
) {
    let popup_height = (state.entries.len() as u16 + 9).clamp(12, 26);
    let popup = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area)[0];
    let popup = Layout::horizontal([Constraint::Percentage(72)])
        .flex(Flex::Center)
        .split(popup)[0];

    f.render_widget(Clear, popup);
    let block = theme::popup_block(" Tag Manager ");
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    let header = vec![
        Line::from("j/k: move    n: new tag"),
        Line::from("Enter: new when empty    r: rename global"),
        Line::from("Esc: close"),
    ];
    f.render_widget(
        Paragraph::new(header)
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        chunks[0],
    );

    let items = if state.entries.is_empty() {
        vec![ListItem::new(Line::from(
            "tag がありません。n で新規作成してください。",
        ))]
    } else {
        state
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let style = if index == state.selected {
                    theme::popup_highlight()
                } else {
                    theme::popup()
                };
                ListItem::new(Line::from(Span::styled(
                    entry.tag.clone(),
                    style.fg(theme::CYAN),
                )))
            })
            .collect::<Vec<_>>()
    };
    f.render_widget(List::new(items).style(theme::popup()), chunks[1]);

    f.render_widget(
        Paragraph::new(format!("{} tags", state.entries.len())).style(theme::popup_soft()),
        chunks[2],
    );
}

pub(in crate::ui) fn render_group_manager(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &GroupManagerState,
) {
    let popup_height = (state.entries.len() as u16 + 9).clamp(12, 26);
    let popup = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area)[0];
    let popup = Layout::horizontal([Constraint::Percentage(72)])
        .flex(Flex::Center)
        .split(popup)[0];

    f.render_widget(Clear, popup);
    let block = theme::popup_block(" Group Manager ");
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    let header = vec![
        Line::from("j/k: move    n: new group"),
        Line::from("Enter: new when empty    r: rename global"),
        Line::from("Esc: close"),
    ];
    f.render_widget(
        Paragraph::new(header)
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        chunks[0],
    );

    let items = if state.entries.is_empty() {
        vec![ListItem::new(Line::from(
            "group がありません。n で新規作成してください。",
        ))]
    } else {
        state
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let style = if index == state.selected {
                    theme::popup_highlight()
                } else {
                    theme::popup()
                };
                ListItem::new(Line::from(Span::styled(
                    entry.group.clone(),
                    style.fg(theme::CYAN),
                )))
            })
            .collect::<Vec<_>>()
    };
    f.render_widget(List::new(items).style(theme::popup()), chunks[1]);

    f.render_widget(
        Paragraph::new(format!("{} groups", state.entries.len())).style(theme::popup_soft()),
        chunks[2],
    );
}
