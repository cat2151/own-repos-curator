use crate::app::{FilterModeFocus, FilterModeState, GroupCatalogState, TagCatalogState};
use crate::ui::{
    layout::{centered_popup_rect, popup_height_for_lines, popup_width_for_lines},
    theme,
};
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

pub(in crate::ui) fn render_filter_mode(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &FilterModeState,
    catalog: &TagCatalogState,
    group_catalog: &GroupCatalogState,
) {
    let mode_label = match state.focus {
        FilterModeFocus::Group => "group",
        FilterModeFocus::Tag => "tag",
    };
    let mut lines = vec![
        Line::from(Span::styled(
            format!("絞り込みモード ({mode_label})"),
            theme::popup_title(),
        )),
        Line::from(match state.focus {
            FilterModeFocus::Group => "a-z でgroup選択 / A-Zどれでもgroup解除",
            FilterModeFocus::Tag => "a-z でtag ON / A-Z でtag OFF",
        }),
        Line::from(match state.focus {
            FilterModeFocus::Group => "Ctrl+T でtagへ切替 / Enter適用 / Esc取消 / ←→でpage移動",
            FilterModeFocus::Tag => "Ctrl+G でgroupへ切替 / Enter適用 / Esc取消 / ←→でpage移動",
        }),
        Line::from(format!(
            "表示予定repo: {}/{}",
            state.visible_repo_count, state.total_repo_count
        )),
        Line::from(""),
        Line::from(Span::styled(
            "現在の絞り込み",
            theme::popup_warning().add_modifier(Modifier::BOLD),
        )),
    ];

    lines.push(Line::from(vec![
        Span::styled("group: ".to_string(), theme::muted()),
        Span::styled(
            state
                .active_group
                .clone()
                .unwrap_or_else(|| "(なし)".to_string()),
            if state.active_group.is_some() {
                theme::popup_warning().add_modifier(Modifier::BOLD)
            } else {
                theme::popup_soft()
            },
        ),
    ]));

    lines.push(Line::from(Span::styled(
        "tags:".to_string(),
        theme::muted(),
    )));
    if state.active_tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ".to_string(), theme::muted()),
            Span::styled("(なし)".to_string(), theme::popup_soft()),
        ]));
    } else {
        let mut active_tag_spans = Vec::new();
        active_tag_spans.push(Span::styled("  ".to_string(), theme::muted()));
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

    match state.focus {
        FilterModeFocus::Group => {
            lines.push(Line::from(format!(
                "group page: {}/{} (total:{})",
                group_catalog.page + 1,
                group_catalog.page_count.max(1),
                group_catalog.total_groups
            )));
            if group_catalog.entries.is_empty() {
                lines.push(Line::from(Span::styled(
                    "登録済みgroup がまだありません。",
                    theme::popup_soft(),
                )));
            } else {
                lines.extend(group_catalog.entries.iter().map(|entry| {
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
        }
        FilterModeFocus::Tag => {
            lines.push(Line::from(format!(
                "tag page: {}/{} (total:{})",
                catalog.page + 1,
                catalog.page_count.max(1),
                catalog.total_tags
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
        }
    }

    let popup_width = popup_width_for_lines(&lines, area, 52, 88);
    let popup_height = popup_height_for_lines(&lines, popup_width, area, 18);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    let block = theme::popup_block(format!(" Filter: {} ", mode_label));
    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(
        Paragraph::new(lines)
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        inner,
    );
}
