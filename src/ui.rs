mod group_colors;
mod help;
mod input_overlays;
mod layout;
mod overlay_binding_modes;
mod overlay_managers;
mod overlays;
mod panels;
mod right_pane;
mod tag_colors;
mod theme;

use self::{
    group_colors::span_for_group,
    help::{
        render_filter_help, render_group_binding_help, render_main_help, render_tag_binding_help,
    },
    input_overlays::{render_group_input, render_tag_input, render_text_editor},
    layout::{left_pane_width, log_pane_height},
    overlay_binding_modes::{render_group_binding_mode, render_tag_binding_mode},
    overlay_managers::{render_group_manager, render_tag_manager},
    overlays::render_filter_mode,
    panels::{
        render_group_summary, render_log_pane, render_selected_repo_desc,
        render_selected_repo_tag_detail, render_tag_catalog,
    },
    right_pane::{layout_right_pane, RightPaneContent},
    tag_colors::span_for_tag,
};
use crate::app::{App, DescDisplayMode, HelpScreen};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

#[derive(Clone, Copy)]
struct RepoItemDisplay<'a> {
    repo_name: &'a str,
    group_name: &'a str,
    registered_groups: &'a [String],
    desc_short: &'a str,
    desc_long: &'a str,
    display_tags: &'a [String],
    registered_tags: &'a [String],
}

pub(crate) fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(Block::default().style(theme::screen()), area);

    let help_screen = app.help_screen();
    let tag_binding_mode = app.tag_binding_mode_state();
    let group_binding_mode = app.group_binding_mode_state();
    let filter_mode = app.filter_mode_state();

    let visible_indices = app.visible_repo_indices();
    let registered_groups = app.registered_groups().to_vec();
    let registered_tags = app.data.registered_tags.clone();
    let group_catalog = app.group_catalog_state();
    let tag_catalog = app.tag_catalog_state();
    let group_summary = app.group_summary_entries();
    let selected_repo_tag_detail = app.selected_repo_tag_detail_state();
    let status_message = app.rendered_status_message();
    let bottom_hint = app.bottom_hint();
    let tag_filter_title = app.tag_filter_title_label();
    let tag_manager = app.tag_manager_state();
    let mut tag_input = app.tag_input.clone();
    let group_manager = app.group_manager_state();
    let mut group_input = app.group_input.clone();
    let debug_log = app.debug_log_lines();
    let desc_display_mode = app.desc_display_mode();
    let debug_log_expanded = app.debug_log_expanded();
    let selected_repo_desc = app.selected_repo_desc_state();
    let group_summary_filtered = app.has_effective_filter();

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
            let display_group = app.display_group_for_repo_index(*index);
            let display_tags = app.display_tags_for_repo_index(*index);
            Some(repo_item_lines(
                RepoItemDisplay {
                    repo_name: repo.name.as_str(),
                    group_name: display_group.as_str(),
                    registered_groups: &registered_groups,
                    desc_short: repo.desc_short.as_str(),
                    desc_long: repo.desc_long.as_str(),
                    display_tags: &display_tags,
                    registered_tags: &registered_tags,
                },
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
        let right_pane = layout_right_pane(
            chunks[1],
            RightPaneContent {
                group_summary: &group_summary,
                group_summary_filtered,
                tag_catalog: &tag_catalog,
                selected_repo_tag_detail: selected_repo_tag_detail.as_ref(),
                selected_repo_desc: selected_repo_desc.as_ref(),
                registered_tags: &registered_tags,
                show_desc: true,
            },
        );

        if right_pane.group_summary.height > 0 {
            render_group_summary(
                f,
                right_pane.group_summary,
                &group_summary,
                &registered_groups,
                group_summary_filtered,
            );
        }
        if right_pane.tag_catalog.height > 0 {
            render_tag_catalog(f, right_pane.tag_catalog, &tag_catalog, &registered_tags);
        }
        if right_pane.tag_detail.height > 0 {
            render_selected_repo_tag_detail(
                f,
                right_pane.tag_detail,
                selected_repo_tag_detail.as_ref(),
                &registered_tags,
            );
        }
        if let Some(desc_area) = right_pane.desc.filter(|area| area.height > 0) {
            render_selected_repo_desc(f, desc_area, selected_repo_desc.as_ref());
        }
    } else {
        let right_pane = layout_right_pane(
            chunks[1],
            RightPaneContent {
                group_summary: &group_summary,
                group_summary_filtered,
                tag_catalog: &tag_catalog,
                selected_repo_tag_detail: selected_repo_tag_detail.as_ref(),
                selected_repo_desc: None,
                registered_tags: &registered_tags,
                show_desc: false,
            },
        );

        if right_pane.group_summary.height > 0 {
            render_group_summary(
                f,
                right_pane.group_summary,
                &group_summary,
                &registered_groups,
                group_summary_filtered,
            );
        }
        if right_pane.tag_catalog.height > 0 {
            render_tag_catalog(f, right_pane.tag_catalog, &tag_catalog, &registered_tags);
        }
        if right_pane.tag_detail.height > 0 {
            render_selected_repo_tag_detail(
                f,
                right_pane.tag_detail,
                selected_repo_tag_detail.as_ref(),
                &registered_tags,
            );
        }
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
    } else if let Some(tag_input) = tag_input.as_mut() {
        render_tag_input(f, screen_chunks[0], tag_input);
    } else if let Some(group_input) = group_input.as_mut() {
        render_group_input(f, screen_chunks[0], group_input);
    } else if let Some(tag_manager) = tag_manager.as_ref() {
        render_tag_manager(f, screen_chunks[0], tag_manager);
    } else if let Some(group_manager) = group_manager.as_ref() {
        render_group_manager(f, screen_chunks[0], group_manager);
    } else if let Some(tag_binding_mode) = tag_binding_mode.as_ref() {
        render_tag_binding_mode(f, screen_chunks[0], tag_binding_mode);
    } else if let Some(group_binding_mode) = group_binding_mode.as_ref() {
        render_group_binding_mode(f, screen_chunks[0], group_binding_mode, &group_catalog);
    } else if let Some(filter_mode) = filter_mode.as_ref() {
        render_filter_mode(
            f,
            screen_chunks[0],
            filter_mode,
            &tag_catalog,
            &group_catalog,
        );
    }

    if matches!(help_screen, Some(HelpScreen::TagBinding)) {
        render_tag_binding_help(f, area, tag_binding_mode.as_ref());
    } else if matches!(help_screen, Some(HelpScreen::GroupBinding)) {
        render_group_binding_help(f, area, group_binding_mode.as_ref());
    } else if matches!(help_screen, Some(HelpScreen::Filter)) {
        render_filter_help(f, area, filter_mode.as_ref());
    } else if matches!(help_screen, Some(HelpScreen::Main)) {
        render_main_help(f, area);
    }
}

fn repo_line(
    repo_name: &str,
    group_name: &str,
    registered_groups: &[String],
    display_tags: &[String],
    registered_tags: &[String],
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(repo_name.to_string(), theme::accent()),
        Span::styled(" <".to_string(), theme::soft()),
        span_for_group(registered_groups, group_name),
        Span::styled(">".to_string(), theme::soft()),
    ];

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
    repo: RepoItemDisplay<'_>,
    desc_display_mode: DescDisplayMode,
) -> Vec<Line<'static>> {
    let mut lines = vec![repo_line(
        repo.repo_name,
        repo.group_name,
        repo.registered_groups,
        repo.display_tags,
        repo.registered_tags,
    )];

    let desc_short = repo.desc_short.trim();
    if desc_display_mode.shows_inline_short_desc() && !desc_short.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ".to_string(), theme::muted()),
            Span::styled(desc_short.to_string(), theme::soft()),
        ]));
    }

    let desc_long = repo.desc_long.trim();
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
    use super::{group_colors, repo_item_lines, repo_line, tag_colors, theme, RepoItemDisplay};
    use crate::{app::DescDisplayMode, model::DEFAULT_GROUP_NAME};

    #[test]
    fn repo_line_colors_tags_by_registered_order() {
        let registered_groups = vec!["tools".to_string()];
        let registered_tags = vec!["rust".to_string(), "zig".to_string(), "go".to_string()];
        let display_tags = vec!["go".to_string(), "rust".to_string()];

        let line = repo_line(
            "repo",
            "tools",
            &registered_groups,
            &display_tags,
            &registered_tags,
        );

        assert_eq!(line.spans[5].content.as_ref(), "go");
        assert_eq!(line.spans[5].style, theme::monokai_tag(2));
        assert_eq!(line.spans[7].content.as_ref(), "rust");
        assert_eq!(line.spans[7].style, theme::monokai_tag(0));
    }

    #[test]
    fn repo_line_uses_shared_tag_color_helper() {
        let registered_groups = vec!["tools".to_string()];
        let registered_tags = vec!["rust".to_string(), "zig".to_string()];
        let display_tags = vec!["zig".to_string()];

        let line = repo_line(
            "repo",
            "tools",
            &registered_groups,
            &display_tags,
            &registered_tags,
        );

        assert_eq!(
            line.spans[5].style,
            tag_colors::style_for_tag(&registered_tags, "zig")
        );
    }

    #[test]
    fn repo_line_colors_groups_by_registered_order() {
        let registered_groups = vec![
            "apps".to_string(),
            DEFAULT_GROUP_NAME.to_string(),
            "tools".to_string(),
        ];

        let line = repo_line("repo", "tools", &registered_groups, &[], &[]);

        assert_eq!(line.spans[2].content.as_ref(), "tools");
        assert_eq!(
            line.spans[2].style,
            group_colors::style_for_group(&registered_groups, "tools")
        );
    }

    #[test]
    fn repo_line_renders_default_group_in_light_gray() {
        let registered_groups = vec![DEFAULT_GROUP_NAME.to_string(), "tools".to_string()];

        let line = repo_line("repo", DEFAULT_GROUP_NAME, &registered_groups, &[], &[]);

        assert_eq!(
            line.spans[2].style,
            group_colors::style_for_group(&registered_groups, DEFAULT_GROUP_NAME)
        );
        assert_eq!(line.spans[2].style, theme::monokai_light_gray());
    }

    #[test]
    fn repo_item_lines_include_indented_short_desc_when_inline_mode_is_enabled() {
        let registered_groups = vec!["tools".to_string()];
        let lines = repo_item_lines(
            RepoItemDisplay {
                repo_name: "repo",
                group_name: "tools",
                registered_groups: &registered_groups,
                desc_short: "short desc",
                desc_long: "",
                display_tags: &[],
                registered_tags: &[],
            },
            DescDisplayMode::LeftShort,
        );

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].spans[0].content.as_ref(), "  ");
        assert_eq!(lines[1].spans[1].content.as_ref(), "short desc");
    }

    #[test]
    fn repo_item_lines_omit_short_desc_when_inline_mode_is_disabled() {
        let registered_groups = vec!["tools".to_string()];
        let lines = repo_item_lines(
            RepoItemDisplay {
                repo_name: "repo",
                group_name: "tools",
                registered_groups: &registered_groups,
                desc_short: "short desc",
                desc_long: "",
                display_tags: &[],
                registered_tags: &[],
            },
            DescDisplayMode::RightPane,
        );

        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn repo_item_lines_include_long_desc_when_left_short_and_long_mode_is_enabled() {
        let registered_groups = vec!["tools".to_string()];
        let lines = repo_item_lines(
            RepoItemDisplay {
                repo_name: "repo",
                group_name: "tools",
                registered_groups: &registered_groups,
                desc_short: "short desc",
                desc_long: "line 1\nline 2",
                display_tags: &[],
                registered_tags: &[],
            },
            DescDisplayMode::LeftShortAndLong,
        );

        assert_eq!(lines.len(), 4);
        assert_eq!(lines[2].spans[0].content.as_ref(), "    ");
        assert_eq!(lines[2].spans[1].content.as_ref(), "line 1");
        assert_eq!(lines[3].spans[1].content.as_ref(), "line 2");
    }
}
