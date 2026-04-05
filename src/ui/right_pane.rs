use super::panels::{selected_repo_desc_lines, selected_repo_tag_detail_lines};
use crate::app::{
    GroupSummaryEntry, SelectedRepoDescState, SelectedRepoTagDetailState, TagCatalogState,
};
use ratatui::{layout::Rect, text::Line};

pub(super) struct RightPaneLayout {
    pub(super) group_summary: Rect,
    pub(super) tag_catalog: Rect,
    pub(super) tag_detail: Rect,
    pub(super) desc: Option<Rect>,
}

pub(super) struct RightPaneContent<'a> {
    pub(super) group_summary: &'a [GroupSummaryEntry],
    pub(super) group_summary_filtered: bool,
    pub(super) tag_catalog: &'a TagCatalogState,
    pub(super) selected_repo_tag_detail: Option<&'a SelectedRepoTagDetailState>,
    pub(super) selected_repo_desc: Option<&'a SelectedRepoDescState>,
    pub(super) registered_tags: &'a [String],
    pub(super) show_desc: bool,
}

pub(super) fn layout_right_pane(area: Rect, content: RightPaneContent<'_>) -> RightPaneLayout {
    let mut desired_heights = vec![
        group_summary_desired_height(
            area.width,
            content.group_summary,
            content.group_summary_filtered,
        ),
        tag_catalog_desired_height(area.width, content.tag_catalog),
        selected_repo_tag_detail_desired_height(
            area.width,
            content.selected_repo_tag_detail,
            content.registered_tags,
        ),
    ];
    if content.show_desc {
        desired_heights.push(selected_repo_desc_desired_height(
            area.width,
            content.selected_repo_desc,
        ));
    }

    let heights = packed_heights(area.height, &desired_heights);
    let mut rects = stacked_rects(area, &heights).into_iter();

    RightPaneLayout {
        group_summary: rects.next().unwrap_or_default(),
        tag_catalog: rects.next().unwrap_or_default(),
        tag_detail: rects.next().unwrap_or_default(),
        desc: content.show_desc.then(|| rects.next().unwrap_or_default()),
    }
}

fn group_summary_desired_height(
    area_width: u16,
    entries: &[GroupSummaryEntry],
    filtered_view: bool,
) -> u16 {
    if entries.is_empty() {
        let message = if filtered_view {
            "表示中repoに一致する group 集計はありません。"
        } else {
            "group 集計はまだありません。repo に group を割り当てるとここに表示されます。"
        };
        paragraph_height_for_message(area_width, message)
    } else {
        list_height(entries.len())
    }
}

fn tag_catalog_desired_height(area_width: u16, state: &TagCatalogState) -> u16 {
    if state.entries.is_empty() {
        let message = if state.total_tags == 0 {
            "登録済みtag がまだありません。n で最初のtagを作成できます。"
        } else {
            "この page に表示する登録済みtag がありません。← / → で page を切り替えてください。"
        };
        paragraph_height_for_message(area_width, message)
    } else {
        list_height(state.entries.len())
    }
}

fn selected_repo_tag_detail_desired_height(
    area_width: u16,
    state: Option<&SelectedRepoTagDetailState>,
    registered_tags: &[String],
) -> u16 {
    let Some(state) = state else {
        return paragraph_height_for_message(area_width, "条件に一致するrepoがありません。");
    };
    paragraph_height_for_lines(
        area_width,
        &selected_repo_tag_detail_lines(state, registered_tags),
    )
}

fn selected_repo_desc_desired_height(
    area_width: u16,
    state: Option<&SelectedRepoDescState>,
) -> u16 {
    let Some(state) = state else {
        return paragraph_height_for_message(area_width, "条件に一致するrepoがありません。");
    };
    paragraph_height_for_lines(area_width, &selected_repo_desc_lines(state))
}

fn list_height(item_count: usize) -> u16 {
    (item_count as u16).saturating_add(2)
}

fn paragraph_height_for_message(area_width: u16, message: &str) -> u16 {
    paragraph_height_for_lines(area_width, &[Line::from(message.to_string())])
}

fn paragraph_height_for_lines(area_width: u16, lines: &[Line<'_>]) -> u16 {
    if area_width == 0 {
        return 0;
    }

    let inner_width = usize::from(area_width.saturating_sub(2).max(1));
    let wrapped_line_count = lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(inner_width))
        .sum::<usize>() as u16;

    wrapped_line_count.saturating_add(2)
}

fn packed_heights(total_height: u16, desired_heights: &[u16]) -> Vec<u16> {
    if desired_heights.is_empty() {
        return Vec::new();
    }

    let mut remaining = total_height;
    let mut heights = Vec::with_capacity(desired_heights.len());
    for (index, desired) in desired_heights.iter().copied().enumerate() {
        let height = if index + 1 == desired_heights.len() {
            remaining
        } else {
            desired.min(remaining)
        };
        heights.push(height);
        remaining = remaining.saturating_sub(height);
    }
    heights
}

fn stacked_rects(area: Rect, heights: &[u16]) -> Vec<Rect> {
    let mut offset = 0u16;
    heights
        .iter()
        .copied()
        .map(|requested_height| {
            let height = requested_height.min(area.height.saturating_sub(offset));
            let rect = Rect::new(area.x, area.y.saturating_add(offset), area.width, height);
            offset = offset.saturating_add(height);
            rect
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        group_summary_desired_height, layout_right_pane, packed_heights,
        selected_repo_desc_desired_height, tag_catalog_desired_height,
    };
    use crate::app::{
        GroupSummaryEntry, SelectedRepoDescState, SelectedRepoTagDetailState, TagCatalogState,
    };
    use ratatui::layout::Rect;

    #[test]
    fn group_summary_height_shows_all_groups_without_extra_blank_lines() {
        let height = group_summary_desired_height(
            30,
            &[
                GroupSummaryEntry {
                    group: "apps".to_string(),
                    count: 4,
                },
                GroupSummaryEntry {
                    group: "tools".to_string(),
                    count: 2,
                },
                GroupSummaryEntry {
                    group: "web".to_string(),
                    count: 1,
                },
            ],
            false,
        );

        assert_eq!(height, 5);
    }

    #[test]
    fn tag_catalog_height_uses_message_height_for_empty_state() {
        let height = tag_catalog_desired_height(
            30,
            &TagCatalogState {
                entries: vec![],
                page: 0,
                page_count: 0,
                total_tags: 0,
                active_filter_count: 0,
                filter_mode_active: false,
            },
        );

        assert!(height >= 3);
    }

    #[test]
    fn selected_repo_desc_height_counts_wrapped_lines() {
        let height = selected_repo_desc_desired_height(
            12,
            Some(&SelectedRepoDescState {
                repo_name: "repo-name".to_string(),
                github_desc: "very long github description".to_string(),
                desc_short: "short".to_string(),
                desc_long: "one two three four five".to_string(),
                group: "tools".to_string(),
                group_key_hint: "a".to_string(),
            }),
        );

        assert!(height > 10);
    }

    #[test]
    fn packed_heights_give_remainder_to_last_section() {
        assert_eq!(packed_heights(20, &[5, 4, 3, 2]), vec![5, 4, 3, 8]);
    }

    #[test]
    fn packed_heights_prioritize_upper_sections_when_space_is_tight() {
        assert_eq!(packed_heights(10, &[6, 5, 4]), vec![6, 4, 0]);
    }

    #[test]
    fn layout_right_pane_places_group_summary_above_tag_catalog() {
        let layout = layout_right_pane(
            Rect::new(40, 2, 30, 20),
            super::RightPaneContent {
                group_summary: &[GroupSummaryEntry {
                    group: "tools".to_string(),
                    count: 2,
                }],
                group_summary_filtered: false,
                tag_catalog: &TagCatalogState {
                    entries: vec![],
                    page: 0,
                    page_count: 0,
                    total_tags: 0,
                    active_filter_count: 0,
                    filter_mode_active: false,
                },
                selected_repo_tag_detail: Some(&SelectedRepoTagDetailState {
                    repo_name: "repo".to_string(),
                    tag_count: 0,
                    entries: vec![],
                }),
                selected_repo_desc: None,
                registered_tags: &[],
                show_desc: false,
            },
        );

        assert_eq!(layout.group_summary.y, 2);
        assert!(layout.tag_catalog.y >= layout.group_summary.bottom());
        assert!(layout.tag_detail.y >= layout.tag_catalog.bottom());
    }
}
