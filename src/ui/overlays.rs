use super::{
    help::{group_binding_mode_keybind_lines, tag_binding_mode_keybind_lines},
    layout::{centered_popup_rect, popup_height_for_lines, popup_width_for_lines},
    theme,
};
use crate::app::{
    GroupBindingModeState, GroupCatalogState, GroupManagerState, TagBindingModeState,
    TagCatalogState, TagFilterModeState, TagManagerState,
};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, Wrap},
};

#[cfg(test)]
mod tests;

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

pub(super) fn render_tag_binding_mode(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &TagBindingModeState,
) {
    let mut lines = vec![
        Line::from(Span::styled(
            "通常画面のtag紐付けモード",
            theme::popup_title(),
        )),
        Line::from("tag変更はまだ保存されていません。"),
        Line::from(""),
        Line::from(Span::styled(
            "操作キー",
            theme::popup_warning().add_modifier(Modifier::BOLD),
        )),
    ];
    lines.extend(
        tag_binding_mode_keybind_lines()
            .into_iter()
            .map(|line| line.style(theme::popup_soft())),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(format!("現在のtag数: {}", state.pending_count)));

    if !state.added_tags.is_empty() {
        lines.push(Line::from(format!(
            "追加予定: {}",
            state.added_tags.join(", ")
        )));
    }
    if !state.removed_tags.is_empty() {
        lines.push(Line::from(format!(
            "削除予定: {}",
            state.removed_tags.join(", ")
        )));
    }
    if state.added_tags.is_empty() && state.removed_tags.is_empty() {
        lines.push(Line::from(Span::styled(
            "未確定の変更はありません。",
            theme::popup_soft(),
        )));
    }

    let popup_width = popup_width_for_lines(&lines, area, 48, 84);
    let popup_height = popup_height_for_lines(&lines, popup_width, area, 16);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    let block = theme::popup_block(format!(" Tag Bind Mode: {} ", state.repo_name));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    f.render_widget(
        Paragraph::new(lines)
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        inner,
    );
}

pub(super) fn render_group_binding_mode(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &GroupBindingModeState,
    catalog: &GroupCatalogState,
) {
    let mut lines = vec![
        Line::from(Span::styled(
            "通常画面のgroup割り当てモード",
            theme::popup_title(),
        )),
        Line::from("表示中の小文字キーを押すと即時確定します。"),
        Line::from(""),
        Line::from(Span::styled(
            "操作キー",
            theme::popup_warning().add_modifier(Modifier::BOLD),
        )),
    ];
    lines.extend(
        group_binding_mode_keybind_lines()
            .into_iter()
            .map(|line| line.style(theme::popup_soft())),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(format!("現在: {}", state.original_group)));
    lines.push(Line::from(format!("選択候補: {}", state.pending_group)));
    lines.push(Line::from(format!(
        "group page: {}/{} (total:{})",
        catalog.page + 1,
        catalog.page_count.max(1),
        catalog.total_groups
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "このpageのショートカット",
        theme::popup_warning().add_modifier(Modifier::BOLD),
    )));

    if catalog.entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "登録済みgroup がまだありません。Ctrl+G で作成できます。",
            theme::popup_soft(),
        )));
    } else {
        lines.extend(catalog.entries.iter().map(|entry| {
            let marker = if entry.selected { "[x]" } else { "[ ]" };
            let marker_style = if entry.selected {
                theme::popup_warning().add_modifier(Modifier::BOLD)
            } else {
                theme::popup_soft()
            };
            Line::from(vec![
                Span::styled(format!("{} ", entry.key), theme::popup_title()),
                Span::styled(format!("{marker} "), marker_style),
                Span::styled(entry.group.clone(), theme::popup()),
            ])
        }));
    }

    let popup_width = popup_width_for_lines(&lines, area, 52, 88);
    let popup_height = popup_height_for_lines(&lines, popup_width, area, 18);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    let block = theme::popup_block(format!(" Group Bind Mode: {} ", state.repo_name));
    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(
        Paragraph::new(lines)
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        inner,
    );
}

pub(super) fn render_tag_filter_mode(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &TagFilterModeState,
    catalog: &TagCatalogState,
) {
    let mut lines = vec![
        Line::from(Span::styled("tag絞り込みモード", theme::popup_title())),
        Line::from("a-z で候補 ON / A-Z で候補 OFF"),
        Line::from("Enter で適用 / Esc で取消 / ←→ で page移動"),
        Line::from(format!(
            "表示予定repo: {}/{}",
            state.visible_repo_count, state.total_repo_count
        )),
        Line::from(format!(
            "tag page: {}/{} (total:{})",
            catalog.page + 1,
            catalog.page_count.max(1),
            catalog.total_tags
        )),
        Line::from(""),
        Line::from(Span::styled(
            "現在の絞り込み候補",
            theme::popup_warning().add_modifier(Modifier::BOLD),
        )),
    ];

    if state.active_tags.is_empty() {
        lines.push(Line::from(Span::styled("(なし)", theme::popup_soft())));
    } else {
        let mut active_tag_spans = Vec::new();
        for (index, tag) in state.active_tags.iter().enumerate() {
            if index > 0 {
                active_tag_spans.push(Span::styled(", ", theme::popup_soft()));
            }
            active_tag_spans.push(Span::styled(
                tag.clone(),
                theme::popup_warning().add_modifier(Modifier::BOLD),
            ));
        }
        lines.push(Line::from(active_tag_spans));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "このpageのショートカット",
        theme::popup_warning().add_modifier(Modifier::BOLD),
    )));

    if catalog.entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "登録済みtag がまだありません。n で作成できます。",
            theme::popup_soft(),
        )));
    } else {
        lines.extend(catalog.entries.iter().map(|entry| {
            let state_label = if entry.filter_active { "ON " } else { "OFF" };
            let state_style = if entry.filter_active {
                theme::popup_warning().add_modifier(Modifier::BOLD)
            } else {
                theme::popup_soft()
            };
            Line::from(vec![
                Span::styled(
                    format!("{}/{} ", entry.key, entry.key.to_ascii_uppercase()),
                    theme::popup_title(),
                ),
                Span::styled(format!("[{state_label}] "), state_style),
                Span::styled(entry.tag.clone(), theme::popup()),
            ])
        }));
    }

    let popup_width = popup_width_for_lines(&lines, area, 52, 88);
    let popup_height = popup_height_for_lines(&lines, popup_width, area, 18);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    let block = theme::popup_block(" Tag Filter ");
    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(
        Paragraph::new(lines)
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        inner,
    );
}
