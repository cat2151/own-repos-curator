use super::{
    help::{group_binding_mode_keybind_lines, tag_binding_mode_keybind_lines},
    layout::{centered_popup_rect, popup_height_for_lines, popup_width_for_lines},
    theme,
};
use crate::app::{
    FilterModeFocus, FilterModeState, GroupBindingModeState, GroupCatalogState, GroupManagerState,
    TagBindingModeState, TagCatalogState, TagManagerState,
};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Modifier,
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

pub(super) fn render_filter_mode(
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

#[cfg(test)]
mod tests {
    use super::{render_filter_mode, render_group_binding_mode};
    use crate::app::{
        FilterModeFocus, FilterModeState, GroupBindingModeState, GroupCatalogEntry,
        GroupCatalogState, TagCatalogEntry, TagCatalogState,
    };
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    fn render_filter_overlay_text(
        width: u16,
        height: u16,
        state: &FilterModeState,
        catalog: &TagCatalogState,
        group_catalog: &GroupCatalogState,
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|f| {
                render_filter_mode(
                    f,
                    Rect::new(0, 0, width, height),
                    state,
                    catalog,
                    group_catalog,
                );
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_group_binding_overlay_text(
        width: u16,
        height: u16,
        state: &GroupBindingModeState,
        catalog: &GroupCatalogState,
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|f| {
                render_group_binding_mode(f, Rect::new(0, 0, width, height), state, catalog);
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn tag_filter_overlay_renders_summary_and_shortcuts() {
        let state = FilterModeState {
            focus: FilterModeFocus::Tag,
            active_group: Some("tools".to_string()),
            active_tags: vec!["rust".to_string()],
            visible_repo_count: 3,
            total_repo_count: 7,
        };
        let catalog = TagCatalogState {
            entries: vec![
                TagCatalogEntry {
                    key: 'r',
                    filter_active: true,
                    tag: "rust".to_string(),
                },
                TagCatalogEntry {
                    key: 'g',
                    filter_active: false,
                    tag: "go".to_string(),
                },
            ],
            page: 0,
            page_count: 1,
            total_tags: 2,
            active_filter_count: 1,
            filter_mode_active: true,
        };
        let group_catalog = GroupCatalogState {
            entries: vec![
                GroupCatalogEntry {
                    key: 't',
                    selected: true,
                    group: "tools".to_string(),
                },
                GroupCatalogEntry {
                    key: 'w',
                    selected: false,
                    group: "web".to_string(),
                },
            ],
            page: 0,
            page_count: 1,
            total_groups: 2,
        };

        let rendered = render_filter_overlay_text(80, 24, &state, &catalog, &group_catalog);

        assert!(rendered.contains("Filter: tag"));
        assert!(rendered.contains("3/7"));
        assert!(rendered.contains("tools"));
        assert!(rendered.contains("rust"));
        assert!(rendered.contains("r/R"));
        assert!(rendered.contains("[ON ]"));
    }

    #[test]
    fn group_filter_overlay_renders_summary_and_shortcuts() {
        let state = FilterModeState {
            focus: FilterModeFocus::Group,
            active_group: Some("web".to_string()),
            active_tags: vec!["rust".to_string()],
            visible_repo_count: 2,
            total_repo_count: 7,
        };
        let catalog = TagCatalogState {
            entries: vec![],
            page: 0,
            page_count: 0,
            total_tags: 0,
            active_filter_count: 1,
            filter_mode_active: true,
        };
        let group_catalog = GroupCatalogState {
            entries: vec![
                GroupCatalogEntry {
                    key: 't',
                    selected: false,
                    group: "tools".to_string(),
                },
                GroupCatalogEntry {
                    key: 'w',
                    selected: true,
                    group: "web".to_string(),
                },
            ],
            page: 0,
            page_count: 1,
            total_groups: 2,
        };

        let rendered = render_filter_overlay_text(80, 24, &state, &catalog, &group_catalog);

        assert!(rendered.contains("Filter: group"));
        assert!(rendered.contains("2/7"));
        assert!(rendered.contains("Ctrl+T"));
        assert!(rendered.contains("web"));
        assert!(rendered.contains("[x]"));
    }

    #[test]
    fn group_binding_overlay_renders_summary_and_shortcuts() {
        let state = GroupBindingModeState {
            repo_name: "selected".to_string(),
            original_group: "tools".to_string(),
            pending_group: "web".to_string(),
        };
        let catalog = GroupCatalogState {
            entries: vec![
                GroupCatalogEntry {
                    key: 't',
                    selected: false,
                    group: "tools".to_string(),
                },
                GroupCatalogEntry {
                    key: 'w',
                    selected: true,
                    group: "web".to_string(),
                },
            ],
            page: 0,
            page_count: 1,
            total_groups: 2,
        };

        let rendered = render_group_binding_overlay_text(80, 24, &state, &catalog);

        assert!(rendered.contains("Group Bind Mode"));
        assert!(rendered.contains("selected"));
        assert!(rendered.contains("tools"));
        assert!(rendered.contains("web"));
        assert!(rendered.contains("w"));
        assert!(rendered.contains("[x]"));
    }
}
