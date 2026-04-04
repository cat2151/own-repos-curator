use super::{
    help::tag_binding_mode_keybind_lines,
    layout::{centered_popup_rect, popup_height_for_lines, popup_width_for_lines},
    theme,
};
use crate::app::{
    EditorField, TagBindingModeState, TagCatalogState, TagFilterModeState, TagInput, TagInputMode,
    TagManagerState, TextEditor,
};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, Wrap},
};

pub(super) fn render_text_editor(f: &mut ratatui::Frame, area: Rect, editor: &mut TextEditor) {
    let popup_height = match editor.field {
        EditorField::ShortDesc => 7,
        EditorField::LongDesc => 11,
    };
    let popup = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area)[0];
    let popup = Layout::horizontal([Constraint::Percentage(70)])
        .flex(Flex::Center)
        .split(popup)[0];

    f.render_widget(Clear, popup);
    let block = theme::popup_block(match editor.field {
        EditorField::ShortDesc => " 1行説明を編集 ",
        EditorField::LongDesc => " 3行説明を編集 ",
    });
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    editor.textarea.remove_block();
    editor.textarea.set_style(theme::popup());
    editor.textarea.set_cursor_style(theme::popup_highlight());
    editor.textarea.set_cursor_line_style(Style::default());
    editor.textarea.set_placeholder_style(theme::popup_muted());
    editor.textarea.set_placeholder_text(match editor.field {
        EditorField::ShortDesc => "1行説明を入力",
        EditorField::LongDesc => "3行説明を入力",
    });

    f.render_widget(&editor.textarea, inner_chunks[0]);
    f.render_widget(
        Paragraph::new(match editor.field {
            EditorField::ShortDesc => "Enter: 保存    Esc: キャンセル    ←→/Home/End: 移動",
            EditorField::LongDesc => "Ctrl+S: 保存    Enter: 改行    Esc: キャンセル",
        })
        .style(theme::popup_soft()),
        inner_chunks[1],
    );
}

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

pub(super) fn render_tag_input(f: &mut ratatui::Frame, area: Rect, input: &TagInput) {
    let popup = Layout::vertical([Constraint::Length(7)])
        .flex(Flex::Center)
        .split(area)[0];
    let popup = Layout::horizontal([Constraint::Percentage(55)])
        .flex(Flex::Center)
        .split(popup)[0];

    f.render_widget(Clear, popup);
    let title = match &input.mode {
        TagInputMode::CreateAndAssignToSelectedRepo | TagInputMode::CreateRegisteredOnly => {
            " 新規tag "
        }
        TagInputMode::RenameGlobal { .. } => " tag rename ",
    };
    let block = theme::popup_block(title);
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .split(inner);

    let input_line = if input.buffer.is_empty() {
        Line::from(Span::styled(
            "新しいtag名を入力してください。",
            theme::popup_soft(),
        ))
    } else {
        Line::from(Span::styled(input.buffer.as_str(), theme::popup()))
    };
    f.render_widget(
        Paragraph::new(vec![input_line, Line::from("")])
            .style(theme::popup())
            .wrap(Wrap { trim: false }),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new("Enter: 保存    Esc: キャンセル").style(theme::popup_soft()),
        chunks[1],
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

#[cfg(test)]
mod tests {
    use super::render_tag_filter_mode;
    use crate::app::{TagCatalogEntry, TagCatalogState, TagFilterModeState};
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    fn render_tag_filter_overlay_text(
        width: u16,
        height: u16,
        state: &TagFilterModeState,
        catalog: &TagCatalogState,
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|f| {
                render_tag_filter_mode(f, Rect::new(0, 0, width, height), state, catalog);
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
        let state = TagFilterModeState {
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

        let rendered = render_tag_filter_overlay_text(80, 24, &state, &catalog);

        assert!(rendered.contains("Tag Filter"));
        assert!(rendered.contains("3/7"));
        assert!(rendered.contains("rust"));
        assert!(rendered.contains("r/R"));
        assert!(rendered.contains("[ON ]"));
    }
}
