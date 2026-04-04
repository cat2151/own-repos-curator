use super::super::super::{
    helpers::{summarize_tag_counts, tag_bindings_for_page, tag_page_count, tag_shortcut_for_tag},
    TagSummaryEntry, TAG_KEYS,
};

#[test]
fn tag_page_count_rounds_up() {
    assert_eq!(tag_page_count(0), 0);
    assert_eq!(tag_page_count(1), 1);
    assert_eq!(tag_page_count(TAG_KEYS.len()), 1);
    assert_eq!(tag_page_count(TAG_KEYS.len() + 1), 2);
    assert_eq!(tag_page_count(TAG_KEYS.len() * 2 + 3), 3);
}

#[test]
fn tag_bindings_prefer_tag_initials_on_each_page() {
    let mut tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    tags.extend(["rust", "vibes", "zig"].map(String::from));

    let bindings = tag_bindings_for_page(&tags, 1);

    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings[0].key, 'r');
    assert_eq!(bindings[0].tag, "rust");
    assert_eq!(bindings[1].key, 'v');
    assert_eq!(bindings[2].key, 'z');
}

#[test]
fn tag_bindings_clamp_to_last_page() {
    let mut tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    tags.extend(["rust", "vibes"].map(String::from));

    let bindings = tag_bindings_for_page(&tags, 99);

    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].key, 'r');
    assert_eq!(bindings[0].tag, "rust");
    assert_eq!(bindings[1].key, 'v');
    assert_eq!(bindings[1].tag, "vibes");
}

#[test]
fn tag_shortcut_for_tag_returns_page_and_key() {
    let mut tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    tags.extend(["rust", "vibes"].map(String::from));

    assert_eq!(tag_shortcut_for_tag(&tags, "a-page-0"), Some((0, 'a')));
    assert_eq!(tag_shortcut_for_tag(&tags, "rust"), Some((1, 'r')));
    assert_eq!(tag_shortcut_for_tag(&tags, "missing"), None);
}

#[test]
fn tag_bindings_fall_back_to_next_available_letter_when_initials_collide() {
    let tags = vec![
        "rust".to_string(),
        "ruby".to_string(),
        "rock".to_string(),
        "vibes".to_string(),
    ];

    let bindings = tag_bindings_for_page(&tags, 0);

    assert_eq!(bindings[0].key, 'r');
    assert_eq!(bindings[0].tag, "rust");
    assert_eq!(bindings[1].key, 'u');
    assert_eq!(bindings[1].tag, "ruby");
    assert_eq!(bindings[2].key, 'o');
    assert_eq!(bindings[2].tag, "rock");
    assert_eq!(bindings[3].key, 'v');
    assert_eq!(bindings[3].tag, "vibes");
}

#[test]
fn summarize_tag_counts_aggregates_and_sorts_by_count_then_name() {
    let repo_a = vec!["rust".to_string(), "tui".to_string()];
    let repo_b = vec!["rust".to_string(), "obsidian plugin".to_string()];
    let repo_c = vec!["obsidian plugin".to_string()];

    let summary = summarize_tag_counts([repo_a.as_slice(), repo_b.as_slice(), repo_c.as_slice()]);

    assert_eq!(
        summary,
        vec![
            TagSummaryEntry {
                tag: "obsidian plugin".to_string(),
                count: 2,
            },
            TagSummaryEntry {
                tag: "rust".to_string(),
                count: 2,
            },
            TagSummaryEntry {
                tag: "tui".to_string(),
                count: 1,
            },
        ]
    );
}

#[test]
fn summarize_tag_counts_returns_empty_for_empty_input() {
    let summary = summarize_tag_counts(std::iter::empty::<&[String]>());

    assert!(summary.is_empty());
}
