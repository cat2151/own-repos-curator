use super::theme;
use crate::app::{GroupManagerState, TagManagerState};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, Wrap},
};

pub(super) fn render_tag_manager(f: &mut ratatui::Frame, area: Rect, state: &TagManagerState) {
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

pub(super) fn render_group_manager(f: &mut ratatui::Frame, area: Rect, state: &GroupManagerState) {
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

#[cfg(test)]
mod tests {
    use super::{render_group_manager, render_tag_manager};
    use crate::app::{GroupManagerState, TagManagerEntry, TagManagerState};
    use crate::ui::test_utils::render_overlay_text;
    use ratatui::layout::Rect;

    #[test]
    fn tag_manager_overlay_renders_entries_and_count() {
        let state = TagManagerState {
            entries: vec![
                TagManagerEntry {
                    tag: "rust".to_string(),
                },
                TagManagerEntry {
                    tag: "go".to_string(),
                },
            ],
            selected: 1,
        };

        let rendered = render_overlay_text(80, 24, |f| {
            render_tag_manager(f, Rect::new(0, 0, 80, 24), &state);
        });

        assert!(rendered.contains("Tag Manager"));
        assert!(rendered.contains("rust"));
        assert!(rendered.contains("go"));
        assert!(rendered.contains("2 tags"));
    }

    #[test]
    fn group_manager_overlay_renders_empty_state_message() {
        let state = GroupManagerState {
            entries: vec![],
            selected: 0,
        };

        let rendered = render_overlay_text(80, 24, |f| {
            render_group_manager(f, Rect::new(0, 0, 80, 24), &state);
        });

        assert!(rendered.contains("Group Manager"));
        assert!(rendered.contains("n: new group"));
        assert!(rendered.contains("0 groups"));
    }
}
