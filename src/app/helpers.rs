use super::{SortMode, TagBinding, TagSummaryEntry, TAG_KEYS};
use crate::model::Repo;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashSet},
};

pub(crate) fn tag_page_count(total_tags: usize) -> usize {
    if total_tags == 0 {
        0
    } else {
        1 + (total_tags - 1) / TAG_KEYS.len()
    }
}

pub(crate) fn clamp_tag_page(page: usize, page_count: usize) -> usize {
    if page_count == 0 {
        0
    } else {
        page.min(page_count - 1)
    }
}

pub(crate) fn tag_bindings_for_page(registered_tags: &[String], page: usize) -> Vec<TagBinding> {
    let page_count = tag_page_count(registered_tags.len());
    let page = clamp_tag_page(page, page_count);
    let start = page.saturating_mul(TAG_KEYS.len());
    let page_tags = registered_tags
        .iter()
        .skip(start)
        .take(TAG_KEYS.len())
        .cloned()
        .collect::<Vec<_>>();
    let mut used_keys = HashSet::with_capacity(TAG_KEYS.len());

    page_tags
        .into_iter()
        .map(|tag| {
            let key = tag_shortcut_candidates(&tag)
                .find(|candidate| used_keys.insert(*candidate))
                .expect("tag shortcut candidates always cover all tag keys");
            TagBinding { key, tag }
        })
        .collect()
}

pub(crate) fn tag_shortcut_for_tag(registered_tags: &[String], tag: &str) -> Option<(usize, char)> {
    let index = registered_tags
        .iter()
        .position(|registered_tag| registered_tag == tag)?;
    let page = index / TAG_KEYS.len();
    let key = tag_bindings_for_page(registered_tags, page)
        .into_iter()
        .find(|binding| binding.tag == tag)
        .map(|binding| binding.key)?;
    Some((page, key))
}

fn tag_shortcut_candidates(tag: &str) -> impl Iterator<Item = char> {
    let mut seen = HashSet::with_capacity(TAG_KEYS.len());
    let mut candidates = tag
        .chars()
        .flat_map(|ch| ch.to_lowercase())
        .filter(|ch| ch.is_ascii_alphabetic())
        .filter(|ch| seen.insert(*ch))
        .collect::<Vec<_>>();

    candidates.extend(TAG_KEYS.into_iter().filter(|ch| seen.insert(*ch)));
    candidates.into_iter()
}

pub(crate) fn summarize_tag_counts<'a, I>(tag_lists: I) -> Vec<TagSummaryEntry>
where
    I: IntoIterator<Item = &'a [String]>,
{
    let mut counts = BTreeMap::new();
    for tags in tag_lists {
        for tag in tags {
            *counts.entry(tag.clone()).or_insert(0usize) += 1;
        }
    }

    let mut entries = counts
        .into_iter()
        .map(|(tag, count)| TagSummaryEntry { tag, count })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.tag.cmp(&right.tag))
    });
    entries
}

pub(crate) fn sort_repo_indices(indices: &mut [usize], repos: &[Repo], sort_mode: SortMode) {
    indices.sort_by(|left, right| compare_repos(&repos[*left], &repos[*right], sort_mode));
}

fn compare_repos(left: &Repo, right: &Repo, sort_mode: SortMode) -> Ordering {
    match sort_mode {
        SortMode::Created => right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| {
                right
                    .updated_at_or_created()
                    .cmp(left.updated_at_or_created())
            })
            .then_with(|| left.name.cmp(&right.name)),
        SortMode::Modified => right
            .updated_at_or_created()
            .cmp(left.updated_at_or_created())
            .then_with(|| right.created_at.cmp(&left.created_at))
            .then_with(|| left.name.cmp(&right.name)),
    }
}

pub(crate) fn normalize_tags(tags: &mut Vec<String>) {
    *tags = tags
        .iter()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .map(|tag| tag.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
}

pub(crate) fn describe_key_code(code: &KeyCode) -> String {
    match code {
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Char(ch) => format!("Char({:?})", ch),
        other => format!("{other:?}"),
    }
}

pub(crate) fn is_ctrl_char(key: &KeyEvent, ch: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(
            key.code,
            KeyCode::Char(code) if code.eq_ignore_ascii_case(&ch)
        )
}

pub(crate) fn is_plain_char(key: &KeyEvent, ch: char) -> bool {
    !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
        && matches!(
            key.code,
            KeyCode::Char(code) if code.eq_ignore_ascii_case(&ch)
        )
}

pub(crate) fn is_plain_or_ctrl_char(key: &KeyEvent, ch: char) -> bool {
    is_plain_char(key, ch) || is_ctrl_char(key, ch)
}

pub(crate) fn is_shift_char(key: &KeyEvent, ch: char) -> bool {
    key.modifiers.contains(KeyModifiers::SHIFT)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
        && matches!(
            key.code,
            KeyCode::Char(code) if code.eq_ignore_ascii_case(&ch)
        )
}

pub(crate) fn is_quit_key(key: &KeyEvent) -> bool {
    is_plain_char(key, 'q') || is_ctrl_char(key, 'q')
}
