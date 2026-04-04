use super::theme;
use crate::app::{EditorField, GroupInput, GroupInputMode, TagInput, TagInputMode, TextEditor};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Style,
    widgets::{Clear, Paragraph},
};
use tui_textarea::TextArea;

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

    style_popup_textarea(
        &mut editor.textarea,
        match editor.field {
            EditorField::ShortDesc => "1行説明を入力",
            EditorField::LongDesc => "3行説明を入力",
        },
    );

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

pub(super) fn render_tag_input(f: &mut ratatui::Frame, area: Rect, input: &mut TagInput) {
    let title = match &input.mode {
        TagInputMode::CreateAndAssignToSelectedRepo | TagInputMode::CreateRegisteredOnly => {
            " 新規tag "
        }
        TagInputMode::RenameGlobal { .. } => " tag rename ",
    };

    render_single_line_input(
        f,
        area,
        &mut input.textarea,
        title,
        "新しいtag名を入力してください。",
    );
}

pub(super) fn render_group_input(f: &mut ratatui::Frame, area: Rect, input: &mut GroupInput) {
    let title = match &input.mode {
        GroupInputMode::CreateAndAssignToSelectedRepo | GroupInputMode::CreateRegisteredOnly => {
            " 新規group "
        }
        GroupInputMode::RenameGlobal { .. } => " group rename ",
    };

    render_single_line_input(
        f,
        area,
        &mut input.textarea,
        title,
        "新しいgroup名を入力してください。",
    );
}

fn render_single_line_input(
    f: &mut ratatui::Frame,
    area: Rect,
    textarea: &mut TextArea<'static>,
    title: &str,
    placeholder: &str,
) {
    let popup = Layout::vertical([Constraint::Length(7)])
        .flex(Flex::Center)
        .split(area)[0];
    let popup = Layout::horizontal([Constraint::Percentage(55)])
        .flex(Flex::Center)
        .split(popup)[0];

    f.render_widget(Clear, popup);
    let block = theme::popup_block(title);
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    style_popup_textarea(textarea, placeholder);
    f.render_widget(&*textarea, chunks[0]);
    f.render_widget(
        Paragraph::new("Enter/Ctrl+S: 保存    Esc: キャンセル    ←→/Home/End: 移動")
            .style(theme::popup_soft()),
        chunks[1],
    );
}

fn style_popup_textarea(textarea: &mut TextArea<'static>, placeholder: &str) {
    textarea.remove_block();
    textarea.set_style(theme::popup());
    textarea.set_cursor_style(theme::popup_highlight());
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_style(theme::popup_muted());
    textarea.set_placeholder_text(placeholder);
}
