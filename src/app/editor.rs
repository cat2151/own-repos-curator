use super::{App, EditorField, TextEditor};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::{CursorMove, TextArea};

impl TextEditor {
    fn new(field: EditorField, initial_text: &str) -> Self {
        let textarea = new_textarea(initial_text);

        Self { field, textarea }
    }

    fn text_for_save(&self) -> String {
        match self.field {
            EditorField::ShortDesc => textarea_text(&self.textarea, " "),
            EditorField::LongDesc => textarea_text(&self.textarea, "\n"),
        }
    }

    fn normalize_single_line(&mut self) {
        if matches!(self.field, EditorField::ShortDesc) {
            normalize_single_line_textarea(&mut self.textarea);
        }
    }
}

pub(crate) fn new_textarea(initial_text: &str) -> TextArea<'static> {
    configured_textarea(textarea_lines(initial_text))
}

pub(crate) fn new_single_line_textarea(initial_text: &str) -> TextArea<'static> {
    let mut textarea = new_textarea(initial_text);
    normalize_single_line_textarea(&mut textarea);
    textarea
}

pub(crate) fn textarea_text(textarea: &TextArea<'_>, joiner: &str) -> String {
    textarea.lines().join(joiner)
}

pub(crate) fn normalize_single_line_textarea(textarea: &mut TextArea<'static>) {
    if textarea.lines().len() <= 1 {
        return;
    }

    let flattened = textarea_text(textarea, " ");
    *textarea = configured_textarea(vec![flattened]);
}

fn configured_textarea(lines: Vec<String>) -> TextArea<'static> {
    let mut textarea = TextArea::new(lines);
    textarea.set_max_histories(256);
    textarea.move_cursor(CursorMove::Bottom);
    textarea.move_cursor(CursorMove::End);
    textarea
}

fn textarea_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect()
}

impl App {
    pub(crate) fn start_short_desc_edit(&mut self) {
        self.start_editor(EditorField::ShortDesc);
    }

    pub(crate) fn start_long_desc_edit(&mut self) {
        self.start_editor(EditorField::LongDesc);
    }

    pub(crate) fn start_editor(&mut self, field: EditorField) {
        if self.help_screen.is_some() || self.tag_manager.is_some() {
            return;
        }

        let Some(repo) = self.selected_repo() else {
            self.status_message = "編集対象のrepoがありません".to_string();
            return;
        };

        let initial_text = match field {
            EditorField::ShortDesc => repo.desc_short.clone(),
            EditorField::LongDesc => repo.desc_long.clone(),
        };

        self.editor = Some(TextEditor::new(field, &initial_text));
        self.status_message = match field {
            EditorField::ShortDesc => "1行説明を編集中: Enterで保存 / Escでキャンセル".to_string(),
            EditorField::LongDesc => {
                "3行説明を編集中: Ctrl+Sで保存 / Enterで改行 / Escでキャンセル".to_string()
            }
        };
    }

    pub(crate) fn handle_editor_key(&mut self, key: KeyEvent) {
        let Some(field) = self.editor.as_ref().map(|editor| editor.field) else {
            return;
        };

        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.save_editor();
            return;
        }

        match key.code {
            KeyCode::Enter if matches!(field, EditorField::ShortDesc) => self.save_editor(),
            KeyCode::Esc => {
                self.editor = None;
                self.status_message = "編集をキャンセル".to_string();
            }
            _ => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.textarea.input(key);
                    editor.normalize_single_line();
                }
            }
        }
    }

    pub(crate) fn save_editor(&mut self) {
        let Some(index) = self.selected_repo_data_index() else {
            self.editor = None;
            self.status_message = "保存対象のrepoがありません".to_string();
            return;
        };

        let Some(editor) = self.editor.take() else {
            return;
        };

        let repo_name = self.data.repos[index].name.clone();
        let previous = match editor.field {
            EditorField::ShortDesc => self.data.repos[index].desc_short.clone(),
            EditorField::LongDesc => self.data.repos[index].desc_long.clone(),
        };

        let updated_text = editor.text_for_save();
        match editor.field {
            EditorField::ShortDesc => self.data.repos[index].desc_short = updated_text,
            EditorField::LongDesc => self.data.repos[index].desc_long = updated_text,
        }

        match self.persist_data() {
            Ok(()) => {
                let label = match editor.field {
                    EditorField::ShortDesc => "1行説明",
                    EditorField::LongDesc => "3行説明",
                };
                self.status_message = format!("保存完了: {repo_name} の{label}");
            }
            Err(error) => {
                match editor.field {
                    EditorField::ShortDesc => self.data.repos[index].desc_short = previous,
                    EditorField::LongDesc => self.data.repos[index].desc_long = previous,
                }
                self.status_message = format!("保存失敗: {error}");
            }
        }
    }
}
