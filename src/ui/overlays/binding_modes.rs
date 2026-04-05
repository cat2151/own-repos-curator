use crate::app::{GroupBindingModeState, GroupCatalogState, TagBindingModeState};
use crate::ui::{
    help::{group_binding_mode_keybind_lines, tag_binding_mode_keybind_lines},
    layout::{centered_popup_rect, popup_height_for_lines, popup_width_for_lines},
    theme,
};
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

pub(in crate::ui) fn render_tag_binding_mode(
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

pub(in crate::ui) fn render_group_binding_mode(
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
