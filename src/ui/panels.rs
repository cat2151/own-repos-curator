use super::{group_colors, tag_colors, theme};
use crate::app::{
    GroupSummaryEntry, SelectedRepoDescState, SelectedRepoTagDetailState, TagCatalogState,
};
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
};

pub(super) fn render_log_pane(
    f: &mut ratatui::Frame,
    area: Rect,
    status_message: &str,
    debug_log: &[String],
    expanded: bool,
) {
    if area.height == 0 {
        return;
    }

    if !expanded {
        let latest_log = debug_log.last().map(String::as_str);
        let latest_style = if latest_log.is_some() {
            theme::soft()
        } else {
            theme::muted()
        };
        let content = Line::from(vec![
            Span::styled("status: ", theme::muted()),
            Span::styled(status_message.to_string(), theme::soft()),
            Span::styled(" | log: ", theme::muted()),
            Span::styled(latest_log.unwrap_or("debug log is empty"), latest_style),
        ]);

        if area.height < 3 {
            f.render_widget(Paragraph::new(content).style(theme::body()), area);
            return;
        }

        f.render_widget(
            Paragraph::new(vec![content])
                .block(theme::panel_block(" log pane "))
                .style(theme::body())
                .wrap(Wrap { trim: false }),
            area,
        );
        return;
    }

    let visible_log_lines = area.height.saturating_sub(3) as usize;
    let start = debug_log.len().saturating_sub(visible_log_lines);
    let mut lines = vec![Line::from(vec![
        Span::styled("status: ", theme::muted()),
        Span::styled(status_message.to_string(), theme::soft()),
    ])];
    if debug_log.is_empty() {
        lines.push(Line::from(Span::styled(
            "debug log is empty",
            theme::muted(),
        )));
    } else {
        lines.extend(
            debug_log[start..]
                .iter()
                .map(|entry| Line::from(entry.as_str())),
        );
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel_block(" log pane "))
            .style(theme::body())
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub(super) fn render_selected_repo_desc(
    f: &mut ratatui::Frame,
    area: Rect,
    state: Option<&SelectedRepoDescState>,
) {
    let Some(state) = state else {
        f.render_widget(
            Paragraph::new("条件に一致するrepoがありません。")
                .block(theme::panel_block(" 現repoのdesc "))
                .style(theme::soft())
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    };

    f.render_widget(
        Paragraph::new(selected_repo_desc_lines(state))
            .block(theme::panel_block(" 現repoのdesc "))
            .style(theme::body())
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn selected_repo_desc_lines(state: &SelectedRepoDescState) -> Vec<Line<'static>> {
    let mut detail_lines = vec![Line::from(Span::styled(
        state.repo_name.clone(),
        theme::accent().add_modifier(Modifier::BOLD),
    ))];
    detail_lines.push(Line::from(vec![
        Span::styled("group: ".to_string(), theme::muted()),
        Span::styled(state.group.clone(), theme::warning()),
        Span::styled(format!(" ({})", state.group_key_hint), theme::soft()),
    ]));

    detail_lines.push(Line::from(Span::styled(
        "GitHub:".to_string(),
        theme::muted(),
    )));
    if state.github_desc.trim().is_empty() {
        detail_lines.push(Line::from(vec![
            Span::styled("  ".to_string(), theme::muted()),
            Span::styled("(未設定)".to_string(), theme::soft()),
        ]));
    } else {
        detail_lines.extend(state.github_desc.lines().map(|line| {
            Line::from(vec![
                Span::styled("  ".to_string(), theme::muted()),
                Span::styled(line.to_string(), theme::soft()),
            ])
        }));
    }

    if state.desc_short.trim().is_empty() {
        detail_lines.push(Line::from(vec![
            Span::styled("1行: ".to_string(), theme::muted()),
            Span::styled("(未設定)".to_string(), theme::soft()),
        ]));
    } else {
        detail_lines.push(Line::from(vec![
            Span::styled("1行: ".to_string(), theme::muted()),
            Span::styled(state.desc_short.clone(), theme::soft()),
        ]));
    }

    detail_lines.push(Line::from(Span::styled("3行:".to_string(), theme::muted())));
    if state.desc_long.trim().is_empty() {
        detail_lines.push(Line::from(vec![
            Span::styled("  ".to_string(), theme::muted()),
            Span::styled("(未設定)".to_string(), theme::soft()),
        ]));
    } else {
        detail_lines.extend(state.desc_long.lines().map(|line| {
            Line::from(vec![
                Span::styled("  ".to_string(), theme::muted()),
                Span::styled(line.to_string(), theme::soft()),
            ])
        }));
    }

    detail_lines
}

pub(super) fn render_tag_catalog(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &TagCatalogState,
    registered_tags: &[String],
) {
    if state.entries.is_empty() {
        f.render_widget(
            Paragraph::new(if state.total_tags == 0 {
                "登録済みtag がまだありません。n で最初のtagを作成できます。"
            } else {
                "この page に表示する登録済みtag がありません。← / → で page を切り替えてください。"
            })
            .block(theme::panel_block(" 登録済みtag一覧 "))
            .style(theme::soft())
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }

    let items = state
        .entries
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    if entry.filter_active { "[x] " } else { "[ ] " },
                    if entry.filter_active {
                        theme::warning()
                    } else {
                        theme::muted()
                    },
                ),
                Span::styled(
                    format!("{} / {}", entry.key, entry.key.to_ascii_uppercase()),
                    theme::muted(),
                ),
                Span::raw(" "),
                tag_colors::span_for_tag(registered_tags, entry.tag.as_str()),
            ]))
        })
        .collect::<Vec<_>>();
    let title = if state.page_count > 0 {
        format!(
            " 登録済みtag一覧 {}/{}  total:{}  {} ",
            state.page + 1,
            state.page_count,
            state.total_tags,
            tag_filter_label(state.active_filter_count, state.filter_mode_active)
        )
    } else {
        format!(
            " 登録済みtag一覧 {} ",
            tag_filter_label(state.active_filter_count, state.filter_mode_active)
        )
    };
    f.render_widget(
        List::new(items)
            .block(theme::panel_block(title))
            .style(theme::body()),
        area,
    );
}

pub(super) fn render_group_summary(
    f: &mut ratatui::Frame,
    area: Rect,
    entries: &[GroupSummaryEntry],
    registered_groups: &[String],
    filtered_view: bool,
) {
    if entries.is_empty() {
        f.render_widget(
            Paragraph::new(if filtered_view {
                "表示中repoに一致する group 集計はありません。"
            } else {
                "group 集計はまだありません。repo に group を割り当てるとここに表示されます。"
            })
            .block(theme::panel_block(if filtered_view {
                " 表示repo group集計 "
            } else {
                " 全repo group集計 "
            }))
            .style(theme::soft())
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }

    let items = entries
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>3}", entry.count), theme::warning()),
                Span::raw(" "),
                group_colors::span_for_group(registered_groups, entry.group.as_str()),
            ]))
        })
        .collect::<Vec<_>>();
    let title = if filtered_view {
        format!(" 表示repo group集計 unique:{} ", entries.len())
    } else {
        format!(" 全repo group集計 unique:{} ", entries.len())
    };
    f.render_widget(
        List::new(items)
            .block(theme::panel_block(title))
            .style(theme::body()),
        area,
    );
}

fn tag_filter_label(active_count: usize, editing: bool) -> String {
    let prefix = if editing { "filter*" } else { "filter" };
    if active_count == 0 {
        format!("{prefix}:off")
    } else {
        format!("{prefix}:{active_count}")
    }
}

pub(super) fn render_selected_repo_tag_detail(
    f: &mut ratatui::Frame,
    area: Rect,
    state: Option<&SelectedRepoTagDetailState>,
    registered_tags: &[String],
) {
    let Some(state) = state else {
        f.render_widget(
            Paragraph::new("条件に一致するrepoがありません。")
                .block(theme::panel_block(" 現repoのtag詳細 "))
                .style(theme::soft())
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    };

    let mut detail_lines = vec![
        Line::from(Span::styled(
            state.repo_name.as_str(),
            theme::accent().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("tag数: {}", state.tag_count)),
    ];
    if state.entries.is_empty() {
        detail_lines.push(Line::from(Span::styled("(tagなし)", theme::soft())));
    } else {
        detail_lines.extend(state.entries.iter().map(|entry| {
            Line::from(vec![
                Span::styled(format!("{:<11}", entry.key_hint), theme::muted()),
                Span::raw(" "),
                tag_colors::span_for_tag(registered_tags, entry.tag.as_str()),
            ])
        }));
    }

    f.render_widget(
        Paragraph::new(detail_lines)
            .block(theme::panel_block(" 現repoのtag詳細 "))
            .style(theme::body())
            .wrap(Wrap { trim: false }),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::{render_log_pane, selected_repo_desc_lines};
    use crate::app::SelectedRepoDescState;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    fn render_log_pane_lines(
        width: u16,
        height: u16,
        status_message: &str,
        debug_log: &[String],
        expanded: bool,
    ) -> Vec<String> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|f| {
                render_log_pane(
                    f,
                    Rect::new(0, 0, width, height),
                    status_message,
                    debug_log,
                    expanded,
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
            .collect()
    }

    #[test]
    fn collapsed_log_pane_renders_with_a_border_and_status() {
        let lines = render_log_pane_lines(
            48,
            3,
            "保存完了",
            &[String::from("latest log entry")],
            false,
        );

        assert!(lines[0].contains("log pane"));
        assert!(lines[0].starts_with("┌"));
        assert!(lines[1].replace(' ', "").contains("status:保存完了"));
    }

    #[test]
    fn expanded_log_pane_includes_status_and_latest_log_lines() {
        let lines = render_log_pane_lines(
            48,
            6,
            "同期完了",
            &[String::from("log 1"), String::from("log 2")],
            true,
        );

        assert!(lines[1].replace(' ', "").contains("status:同期完了"));
        assert!(lines[2].contains("log 1"));
        assert!(lines[3].contains("log 2"));
    }

    #[test]
    fn selected_repo_desc_pane_includes_github_description() {
        let state = SelectedRepoDescState {
            repo_name: "own-repos-curator".to_string(),
            github_desc: "GitHub managed description".to_string(),
            desc_short: "short desc".to_string(),
            desc_long: "line 1\nline 2".to_string(),
            group: "tools".to_string(),
            group_key_hint: "t".to_string(),
        };

        let lines = selected_repo_desc_lines(&state);

        assert_eq!(lines[1].spans[0].content.as_ref(), "group: ");
        assert_eq!(lines[1].spans[1].content.as_ref(), "tools");
        assert_eq!(lines[2].spans[0].content.as_ref(), "GitHub:");
        assert_eq!(
            lines[3].spans[1].content.as_ref(),
            "GitHub managed description"
        );
        assert_eq!(lines[4].spans[0].content.as_ref(), "1行: ");
        assert_eq!(lines[4].spans[1].content.as_ref(), "short desc");
        assert_eq!(lines[5].spans[0].content.as_ref(), "3行:");
        assert_eq!(lines[6].spans[1].content.as_ref(), "line 1");
        assert_eq!(lines[7].spans[1].content.as_ref(), "line 2");
    }

    #[test]
    fn selected_repo_desc_pane_shows_github_desc_as_unset_when_empty() {
        let state = SelectedRepoDescState {
            repo_name: "own-repos-curator".to_string(),
            github_desc: String::new(),
            desc_short: String::new(),
            desc_long: String::new(),
            group: "tools".to_string(),
            group_key_hint: "t".to_string(),
        };

        let lines = selected_repo_desc_lines(&state);

        assert_eq!(lines[1].spans[0].content.as_ref(), "group: ");
        assert_eq!(lines[2].spans[0].content.as_ref(), "GitHub:");
        assert_eq!(lines[3].spans[1].content.as_ref(), "(未設定)");
        assert_eq!(lines[4].spans[1].content.as_ref(), "(未設定)");
        assert_eq!(lines[6].spans[1].content.as_ref(), "(未設定)");
    }
}
