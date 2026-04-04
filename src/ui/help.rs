use super::layout::{centered_popup_rect, popup_height_for_lines, popup_width_for_lines};
use super::theme;
use crate::app::{GroupBindingModeState, TagBindingModeState};
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

fn tag_binding_shared_keybind_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(" 表示中の小文字キー: 対応tagを紐付け"),
        Line::from(" 表示中の大文字キー: 対応tagを外す"),
        Line::from(" キー割当: 先頭文字優先 / 衝突時は別英字"),
        Line::from(" ← / →  : 登録済みtag page 切替"),
        Line::from(" Enter  : 変更を確定"),
    ]
}

pub(super) fn tag_binding_mode_keybind_lines() -> Vec<Line<'static>> {
    let mut lines = tag_binding_shared_keybind_lines();
    lines.push(Line::from(" Esc    : 変更を破棄して閉じる"));
    lines.push(Line::from(" ?      : 専用helpを開く"));
    lines
}

fn tag_binding_help_keybind_lines() -> Vec<Line<'static>> {
    let mut lines = tag_binding_shared_keybind_lines();
    lines.push(Line::from(" Esc    : このhelpを閉じる"));
    lines.push(Line::from(" ?      : このhelpを閉じる"));
    lines
}

fn group_binding_shared_keybind_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(" 表示中の小文字キー: 対応groupに即時割り当て"),
        Line::from(" キー割当: 先頭文字優先 / 衝突時は別英字"),
        Line::from(" ← / →  : 登録済みgroup page 切替"),
        Line::from(" 小文字キーを押した時点で確定して閉じる"),
    ]
}

pub(super) fn group_binding_mode_keybind_lines() -> Vec<Line<'static>> {
    let mut lines = group_binding_shared_keybind_lines();
    lines.push(Line::from(" Esc    : 変更を破棄して閉じる"));
    lines.push(Line::from(" ?      : 専用helpを開く"));
    lines
}

fn group_binding_help_keybind_lines() -> Vec<Line<'static>> {
    let mut lines = group_binding_shared_keybind_lines();
    lines.push(Line::from(" Esc    : このhelpを閉じる"));
    lines.push(Line::from(" ?      : このhelpを閉じる"));
    lines
}

pub(super) fn render_main_help(f: &mut ratatui::Frame, area: Rect) {
    let help_text = vec![
        Line::from(" t      : tag紐付けモードに入る"),
        Line::from(" mode中 ? : tag紐付けモード専用help"),
        Line::from(" g      : group割り当てモードに入る"),
        Line::from(" mode中 ? : group割り当てモード専用help"),
        Line::from(" /      : tag絞り込みモードに入る"),
        Line::from(" mode中 a-z / A-Z : tag絞り込み ON / OFF"),
        Line::from(" Shift+T: tag manager"),
        Line::from(" Shift+G: group manager"),
        Line::from(" Shift+L: debug log 1行/50% 切替"),
        Line::from(" Shift+D: desc 右下/左1行/左1行+3行 循環"),
        Line::from(" ← / →  : 登録済みtag page 切替"),
        Line::from(" e      : 1行説明を編集"),
        Line::from(" l      : 3行説明を編集"),
        Line::from(" n      : 新規tagを追加"),
        Line::from(" Ctrl+G : 新規groupを追加して現repoへ割当"),
        Line::from(" r      : GitHub から repo 一覧を同期"),
        Line::from(" s      : sort create/modify 切替"),
        Line::from(" j / k  : repo選択を移動"),
        Line::from(" ↑ / ↓  : repo選択を移動"),
        Line::from(" ?      : ヘルプ表示/非表示"),
        Line::from(" Esc    : パネルまたはヘルプを閉じる"),
        Line::from(" q      : 終了"),
    ];
    let popup_width = popup_width_for_lines(&help_text, area, 40, 80);
    let popup_height = popup_height_for_lines(&help_text, popup_width, area, 15);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(help_text)
            .block(theme::popup_block(" Help "))
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        popup,
    );
}

pub(super) fn render_tag_binding_help(
    f: &mut ratatui::Frame,
    area: Rect,
    state: Option<&TagBindingModeState>,
) {
    let mut help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "tag紐付けモード help",
            theme::popup_title().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];
    help_text.extend(tag_binding_help_keybind_lines());

    if let Some(state) = state {
        help_text.push(Line::from(""));
        help_text.push(Line::from(format!(" repo   : {}", state.repo_name)));
        help_text.push(Line::from(format!(" tag数  : {}", state.pending_count)));
        if !state.added_tags.is_empty() {
            help_text.push(Line::from(format!(
                " 追加予定: {}",
                state.added_tags.join(", ")
            )));
        }
        if !state.removed_tags.is_empty() {
            help_text.push(Line::from(format!(
                " 削除予定: {}",
                state.removed_tags.join(", ")
            )));
        }
    }

    let popup_width = popup_width_for_lines(&help_text, area, 40, 78);
    let popup_height = popup_height_for_lines(&help_text, popup_width, area, 11);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(help_text)
            .block(theme::popup_block(" Tag Bind Help "))
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        popup,
    );
}

pub(super) fn render_group_binding_help(
    f: &mut ratatui::Frame,
    area: Rect,
    state: Option<&GroupBindingModeState>,
) {
    let mut help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "group割り当てモード help",
            theme::popup_title().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];
    help_text.extend(group_binding_help_keybind_lines());

    if let Some(state) = state {
        help_text.push(Line::from(""));
        help_text.push(Line::from(format!(" repo   : {}", state.repo_name)));
        help_text.push(Line::from(format!(" 現在   : {}", state.original_group)));
        help_text.push(Line::from(format!(" 選択候補: {}", state.pending_group)));
    }

    let popup_width = popup_width_for_lines(&help_text, area, 40, 78);
    let popup_height = popup_height_for_lines(&help_text, popup_width, area, 10);
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(help_text)
            .block(theme::popup_block(" Group Bind Help "))
            .style(theme::popup_soft())
            .wrap(Wrap { trim: false }),
        popup,
    );
}

#[cfg(test)]
mod tests {
    use super::{
        group_binding_help_keybind_lines, group_binding_mode_keybind_lines,
        tag_binding_help_keybind_lines, tag_binding_mode_keybind_lines,
    };
    use ratatui::text::Line;

    fn lines_to_strings(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn tag_binding_mode_keybinds_include_cancel_and_help_actions() {
        let lines = lines_to_strings(&tag_binding_mode_keybind_lines());

        assert!(lines.iter().any(|line| line.contains("小文字キー")));
        assert!(lines.iter().any(|line| line.contains("大文字キー")));
        assert!(lines.iter().any(|line| line.contains("先頭文字優先")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Esc") && line.contains("変更を破棄")));
        assert!(lines
            .iter()
            .any(|line| line.contains("?") && line.contains("専用help")));
    }

    #[test]
    fn tag_binding_help_keybinds_describe_closing_help() {
        let lines = lines_to_strings(&tag_binding_help_keybind_lines());

        assert!(lines.iter().any(|line| line.contains("衝突時は別英字")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Enter") && line.contains("変更を確定")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Esc") && line.contains("このhelp")));
        assert!(lines
            .iter()
            .any(|line| line.contains("?") && line.contains("このhelp")));
    }

    #[test]
    fn group_binding_mode_keybinds_include_assign_and_help_actions() {
        let lines = lines_to_strings(&group_binding_mode_keybind_lines());

        assert!(lines.iter().any(|line| line.contains("即時割り当て")));
        assert!(lines.iter().any(|line| line.contains("先頭文字優先")));
        assert!(!lines.iter().any(|line| line.contains("Enter")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Esc") && line.contains("変更を破棄")));
        assert!(lines
            .iter()
            .any(|line| line.contains("?") && line.contains("専用help")));
    }

    #[test]
    fn group_binding_help_keybinds_describe_immediate_assign_and_close_actions() {
        let lines = lines_to_strings(&group_binding_help_keybind_lines());

        assert!(lines.iter().any(|line| line.contains("衝突時は別英字")));
        assert!(lines
            .iter()
            .any(|line| line.contains("押した時点で確定して閉じる")));
        assert!(lines.iter().all(|line| !line.contains("Enter")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Esc") && line.contains("このhelp")));
    }
}
