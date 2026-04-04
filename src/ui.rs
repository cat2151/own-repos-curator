mod help;
mod layout;
mod overlays;
mod panels;
mod tag_colors;
mod theme;

use self::{
    help::{render_main_help, render_tag_binding_help},
    layout::{left_pane_width, log_pane_height},
    overlays::{
        render_tag_binding_mode, render_tag_filter_mode, render_tag_input, render_tag_manager,
        render_text_editor,
    },
    panels::{
        render_log_pane, render_selected_repo_desc, render_selected_repo_tag_detail,
        render_tag_catalog, render_tag_summary,
    },
    tag_colors::span_for_tag,
};
use crate::app::{App, DescDisplayMode, HelpScreen};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

pub(crate) fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(Block::default().style(theme::screen()), area);

    let help_screen = app.help_screen();
    let tag_binding_mode = app.tag_binding_mode_state();
    let tag_filter_mode = app.tag_filter_mode_state();

    let visible_indices = app.visible_repo_indices();
    let registered_tags = app.data.registered_tags.clone();
    let tag_catalog = app.tag_catalog_state();
    let tag_summary = app.tag_summary_entries();
    let selected_repo_tag_detail = app.selected_repo_tag_detail_state();
    let status_message = app.status_message.clone();
    let bottom_hint = app.bottom_hint();
    let tag_filter_title = app.tag_filter_title_label();
    let tag_manager = app.tag_manager_state();
    let tag_input = app.tag_input.clone();
    let debug_log = app.debug_log_lines();
    let desc_display_mode = app.desc_display_mode();
    let debug_log_expanded = app.debug_log_expanded();
    let selected_repo_desc = app.selected_repo_desc_state();
    let tag_summary_filtered = app.has_effective_tag_filter();

    let screen_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(log_pane_height(area.height, debug_log_expanded)),
            Constraint::Length(1),
        ])
        .split(area);

    let repo_item_lines: Vec<Vec<Line>> = visible_indices
        .iter()
        .filter_map(|index| {
            let repo = app.data.repos.get(*index)?;
            let display_tags = app.display_tags_for_repo_index(*index);
            Some(repo_item_lines(
                repo.name.as_str(),
                repo.desc_short.as_str(),
                repo.desc_long.as_str(),
                &display_tags,
                &registered_tags,
                desc_display_mode,
            ))
        })
        .collect();

    let list_title = format!(
        " own-repos-curator {}/{} | sort:{} | {} ",
        visible_indices.len(),
        app.data.repos.len(),
        app.sort_mode().label(),
        tag_filter_title
    );
    let list_content_width = repo_item_lines
        .iter()
        .flat_map(|lines| lines.iter())
        .map(Line::width)
        .max()
        .unwrap_or(0) as u16;
    let list_title_width = Line::from(list_title.as_str()).width() as u16;
    let desired_left_width = (list_content_width + 4).max(list_title_width + 2);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_pane_width(screen_chunks[0].width, desired_left_width)),
            Constraint::Min(0),
        ])
        .split(screen_chunks[0]);
    let items: Vec<ListItem> = repo_item_lines.into_iter().map(ListItem::new).collect();
    let list = List::new(items)
        .block(theme::panel_block(list_title))
        .style(theme::body())
        .highlight_style(theme::highlight())
        .highlight_symbol("> ");
    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    if desc_display_mode.shows_right_desc_pane() {
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(5),
                Constraint::Fill(3),
                Constraint::Fill(3),
                Constraint::Fill(4),
            ])
            .split(chunks[1]);

        render_tag_catalog(f, right_chunks[0], &tag_catalog, &registered_tags);
        render_tag_summary(
            f,
            right_chunks[1],
            &tag_summary,
            &registered_tags,
            tag_summary_filtered,
        );
        render_selected_repo_tag_detail(
            f,
            right_chunks[2],
            selected_repo_tag_detail.as_ref(),
            &registered_tags,
        );
        render_selected_repo_desc(f, right_chunks[3], selected_repo_desc.as_ref());
    } else {
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(5),
                Constraint::Fill(3),
                Constraint::Fill(3),
            ])
            .split(chunks[1]);

        render_tag_catalog(f, right_chunks[0], &tag_catalog, &registered_tags);
        render_tag_summary(
            f,
            right_chunks[1],
            &tag_summary,
            &registered_tags,
            tag_summary_filtered,
        );
        render_selected_repo_tag_detail(
            f,
            right_chunks[2],
            selected_repo_tag_detail.as_ref(),
            &registered_tags,
        );
    }

    render_log_pane(
        f,
        screen_chunks[1],
        &status_message,
        &debug_log,
        debug_log_expanded,
    );
    f.render_widget(
        Paragraph::new(bottom_hint).style(theme::soft()),
        screen_chunks[2],
    );

    if let Some(editor) = app.editor.as_mut() {
        render_text_editor(f, screen_chunks[0], editor);
    } else if let Some(tag_input) = tag_input.as_ref() {
        render_tag_input(f, screen_chunks[0], tag_input);
    } else if let Some(tag_manager) = tag_manager.as_ref() {
        render_tag_manager(f, screen_chunks[0], tag_manager);
    } else if let Some(tag_binding_mode) = tag_binding_mode.as_ref() {
        render_tag_binding_mode(f, screen_chunks[0], tag_binding_mode);
    } else if let Some(tag_filter_mode) = tag_filter_mode.as_ref() {
        render_tag_filter_mode(f, screen_chunks[0], tag_filter_mode, &tag_catalog);
    }

    if matches!(help_screen, Some(HelpScreen::TagBinding)) {
        render_tag_binding_help(f, area, tag_binding_mode.as_ref());
    } else if matches!(help_screen, Some(HelpScreen::Main)) {
        render_main_help(f, area);
    }
}

fn repo_line(
    repo_name: &str,
    display_tags: &[String],
    registered_tags: &[String],
) -> Line<'static> {
    let mut spans = vec![Span::styled(repo_name.to_string(), theme::accent())];

    if display_tags.is_empty() {
        return Line::from(spans);
    }

    spans.push(Span::styled(" [".to_string(), theme::soft()));
    for (index, tag) in display_tags.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(", ".to_string(), theme::soft()));
        }
        spans.push(span_for_tag(registered_tags, tag));
    }
    spans.push(Span::styled("]".to_string(), theme::soft()));

    Line::from(spans)
}

fn repo_item_lines(
    repo_name: &str,
    desc_short: &str,
    desc_long: &str,
    display_tags: &[String],
    registered_tags: &[String],
    desc_display_mode: DescDisplayMode,
) -> Vec<Line<'static>> {
    let mut lines = vec![repo_line(repo_name, display_tags, registered_tags)];

    let desc_short = desc_short.trim();
    if desc_display_mode.shows_inline_short_desc() && !desc_short.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ".to_string(), theme::muted()),
            Span::styled(desc_short.to_string(), theme::soft()),
        ]));
    }

    let desc_long = desc_long.trim();
    if desc_display_mode.shows_inline_long_desc() && !desc_long.is_empty() {
        lines.extend(desc_long.lines().map(|line| {
            Line::from(vec![
                Span::styled("    ".to_string(), theme::muted()),
                Span::styled(line.to_string(), theme::soft()),
            ])
        }));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{repo_item_lines, repo_line, tag_colors, theme};
    use crate::app::DescDisplayMode;

    #[test]
    fn repo_line_colors_tags_by_registered_order() {
        let registered_tags = vec!["rust".to_string(), "zig".to_string(), "go".to_string()];
        let display_tags = vec!["go".to_string(), "rust".to_string()];

        let line = repo_line("repo", &display_tags, &registered_tags);

        assert_eq!(line.spans[2].content.as_ref(), "go");
        assert_eq!(line.spans[2].style, theme::monokai_tag(2));
        assert_eq!(line.spans[4].content.as_ref(), "rust");
        assert_eq!(line.spans[4].style, theme::monokai_tag(0));
    }

    #[test]
    fn repo_line_uses_shared_tag_color_helper() {
        let registered_tags = vec!["rust".to_string(), "zig".to_string()];
        let display_tags = vec!["zig".to_string()];

        let line = repo_line("repo", &display_tags, &registered_tags);

        assert_eq!(
            line.spans[2].style,
            tag_colors::style_for_tag(&registered_tags, "zig")
        );
    }

    #[test]
    fn repo_item_lines_include_indented_short_desc_when_inline_mode_is_enabled() {
        let lines = repo_item_lines(
            "repo",
            "short desc",
            "",
            &[],
            &[],
            DescDisplayMode::LeftShort,
        );

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].spans[0].content.as_ref(), "  ");
        assert_eq!(lines[1].spans[1].content.as_ref(), "short desc");
    }

    #[test]
    fn repo_item_lines_omit_short_desc_when_inline_mode_is_disabled() {
        let lines = repo_item_lines(
            "repo",
            "short desc",
            "",
            &[],
            &[],
            DescDisplayMode::RightPane,
        );

        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn repo_item_lines_include_long_desc_when_left_short_and_long_mode_is_enabled() {
        let lines = repo_item_lines(
            "repo",
            "short desc",
            "line 1\nline 2",
            &[],
            &[],
            DescDisplayMode::LeftShortAndLong,
        );

        assert_eq!(lines.len(), 4);
        assert_eq!(lines[2].spans[0].content.as_ref(), "    ");
        assert_eq!(lines[2].spans[1].content.as_ref(), "line 1");
        assert_eq!(lines[3].spans[1].content.as_ref(), "line 2");
    }
}
