use super::{render_filter_mode, render_group_binding_mode};
use crate::app::{
    FilterModeFocus, FilterModeState, GroupBindingModeState, GroupCatalogEntry, GroupCatalogState,
    TagCatalogEntry, TagCatalogState,
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
