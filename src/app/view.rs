use super::{
    helpers::{
        clamp_tag_page, sort_repo_indices, summarize_tag_counts, tag_bindings_for_page,
        tag_page_count, tag_shortcut_for_tag,
    },
    App, SelectedRepoDescState, SelectedRepoTagDetailEntry, SelectedRepoTagDetailState,
    TagBindingModeState, TagCatalogEntry, TagCatalogState, TagFilterModeState, TagManagerEntry,
    TagManagerState, TagSummaryEntry,
};
use crate::model::Repo;

impl App {
    pub(crate) fn visible_repo_indices(&self) -> Vec<usize> {
        let mut indices = self
            .data
            .repos
            .iter()
            .enumerate()
            .filter(|(_, repo)| self.repo_matches_effective_tag_filter(repo))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        sort_repo_indices(&mut indices, &self.data.repos, self.sort_mode);
        indices
    }

    pub(crate) fn selected_repo(&self) -> Option<&Repo> {
        self.selected_repo_data_index()
            .and_then(|i| self.data.repos.get(i))
    }

    pub(crate) fn selected_repo_for_display(&self) -> Option<Repo> {
        let mut repo = self.selected_repo()?.clone();
        repo.tags = self.display_tags_for_repo_index(self.selected_repo_data_index()?);
        Some(repo)
    }

    pub(crate) fn display_tags_for_repo_index(&self, index: usize) -> Vec<String> {
        if let Some(mode) = self.tag_binding_mode.as_ref() {
            if mode.repo_index == index {
                return mode.pending_tags.iter().cloned().collect();
            }
        }

        self.data
            .repos
            .get(index)
            .map(|repo| repo.tags.clone())
            .unwrap_or_default()
    }

    pub(crate) fn has_registered_tags(&self) -> bool {
        !self.data.registered_tags.is_empty()
    }

    pub(crate) fn tag_catalog_state(&self) -> TagCatalogState {
        let registered_tags = self.registered_tags();
        let page_count = tag_page_count(registered_tags.len());
        let page = clamp_tag_page(self.registered_tag_page, page_count);
        let active_filter = self.effective_tag_filter();
        let entries = tag_bindings_for_page(registered_tags, page)
            .into_iter()
            .map(|binding| TagCatalogEntry {
                key: binding.key,
                filter_active: active_filter.contains(&binding.tag),
                tag: binding.tag,
            })
            .collect();
        TagCatalogState {
            entries,
            page,
            page_count,
            total_tags: registered_tags.len(),
            active_filter_count: active_filter.len(),
            filter_mode_active: self.tag_filter_mode_active(),
        }
    }

    pub(crate) fn tag_summary_entries(&self) -> Vec<TagSummaryEntry> {
        let visible_indices = self.visible_repo_indices();
        summarize_tag_counts(
            visible_indices
                .iter()
                .filter_map(|index| self.data.repos.get(*index).map(|repo| repo.tags.as_slice())),
        )
    }

    pub(crate) fn tag_manager_state(&self) -> Option<TagManagerState> {
        let manager = self.tag_manager.as_ref()?;
        let entries = self
            .registered_tags()
            .iter()
            .cloned()
            .map(|tag| TagManagerEntry { tag })
            .collect::<Vec<_>>();
        let selected = manager.selected.min(entries.len().saturating_sub(1));
        Some(TagManagerState { entries, selected })
    }

    pub(crate) fn tag_binding_mode_state(&self) -> Option<TagBindingModeState> {
        let mode = self.tag_binding_mode.as_ref()?;
        Some(TagBindingModeState {
            repo_name: mode.repo_name.clone(),
            pending_count: mode.pending_tags.len(),
            added_tags: mode
                .pending_tags
                .difference(&mode.original_tags)
                .cloned()
                .collect(),
            removed_tags: mode
                .original_tags
                .difference(&mode.pending_tags)
                .cloned()
                .collect(),
        })
    }

    pub(crate) fn tag_filter_mode_state(&self) -> Option<TagFilterModeState> {
        self.tag_filter_mode.as_ref()?;
        Some(TagFilterModeState {
            active_tags: self.active_tag_filter_tags(),
            visible_repo_count: self.visible_repo_indices().len(),
            total_repo_count: self.data.repos.len(),
        })
    }

    pub(crate) fn bottom_hint(&self) -> String {
        if self.help_screen.is_some() {
            return "Esc:close ?:close help".to_string();
        }
        if let Some(editor) = self.editor.as_ref() {
            return match editor.field {
                super::EditorField::ShortDesc => "Enter:save Esc:cancel".to_string(),
                super::EditorField::LongDesc => "Ctrl+S:save Esc:cancel".to_string(),
            };
        }
        if self.tag_input.is_some() {
            return "Enter:save Esc:cancel".to_string();
        }
        if self.tag_binding_mode.is_some() {
            return "Enter:save Esc:cancel ?:help".to_string();
        }
        if self.tag_filter_mode_active() {
            return "a-z:on A-Z:off ←→:page Enter:apply Esc:cancel".to_string();
        }
        if self.tag_manager.is_some() {
            return "q:quit Esc:close".to_string();
        }
        "q:quit ?:help".to_string()
    }

    pub(crate) fn selected_repo_tag_detail_state(&self) -> Option<SelectedRepoTagDetailState> {
        let repo = self.selected_repo_for_display()?;
        let registered_tags = self.registered_tags();
        let page_count = tag_page_count(registered_tags.len());
        let entries = repo
            .tags
            .iter()
            .map(|tag| {
                let key_hint = match tag_shortcut_for_tag(registered_tags, tag) {
                    Some((page, key)) if page_count > 1 => format!(
                        "{}/{} ({}/{})",
                        key,
                        key.to_ascii_uppercase(),
                        page + 1,
                        page_count
                    ),
                    Some((_, key)) => format!("{}/{}", key, key.to_ascii_uppercase()),
                    None => "?/?".to_string(),
                };
                SelectedRepoTagDetailEntry {
                    key_hint,
                    tag: tag.clone(),
                }
            })
            .collect();
        Some(SelectedRepoTagDetailState {
            repo_name: repo.name,
            tag_count: repo.tags.len(),
            entries,
        })
    }

    pub(crate) fn selected_repo_desc_state(&self) -> Option<SelectedRepoDescState> {
        let repo = self.selected_repo()?;
        Some(SelectedRepoDescState {
            repo_name: repo.name.clone(),
            github_desc: repo.github_desc.clone(),
            desc_short: repo.desc_short.clone(),
            desc_long: repo.desc_long.clone(),
        })
    }

    pub(crate) fn tag_for_current_page_shortcut(&self, ch: char) -> Option<String> {
        let lowercase = ch.to_ascii_lowercase();
        self.tag_catalog_state()
            .entries
            .into_iter()
            .find(|entry| entry.key == lowercase)
            .map(|entry| entry.tag)
    }

    fn repo_matches_effective_tag_filter(&self, repo: &Repo) -> bool {
        let active_filter = self.effective_tag_filter();
        if active_filter.is_empty() {
            return true;
        }

        active_filter
            .iter()
            .all(|tag| repo.tags.iter().any(|repo_tag| repo_tag == tag))
    }
}
