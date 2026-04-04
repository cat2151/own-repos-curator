use super::{
    helpers::{clamp_tag_page, tag_page_count},
    App,
};

impl App {
    pub(crate) fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub(crate) fn visible_repo_count(&self) -> usize {
        self.visible_repo_indices().len()
    }

    pub(crate) fn selected_repo_data_index(&self) -> Option<usize> {
        let selected = self.selected_index()?;
        self.visible_repo_indices().get(selected).copied()
    }

    pub(crate) fn selected_repo_data_index_mut(&mut self) -> Option<usize> {
        let selected = self.selected_index()?;
        self.visible_repo_indices().get(selected).copied()
    }

    pub(crate) fn registered_tags(&self) -> &[String] {
        &self.data.registered_tags
    }

    pub(crate) fn sync_selection(&mut self) {
        let registered_tag_count = self.registered_tags().len();
        self.registered_tag_page = clamp_tag_page(
            self.registered_tag_page,
            tag_page_count(registered_tag_count),
        );

        if let Some(manager) = self.tag_manager.as_mut() {
            manager.selected = manager.selected.min(registered_tag_count.saturating_sub(1));
        }

        let visible_len = self.visible_repo_count();
        if visible_len == 0 {
            self.list_state.select(None);
            return;
        }

        let next = self.selected_index().unwrap_or(0).min(visible_len - 1);
        self.list_state.select(Some(next));
    }

    pub(crate) fn move_up(&mut self) {
        let visible_len = self.visible_repo_count();
        if visible_len == 0 {
            return;
        }

        let i = match self.selected_index() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub(crate) fn move_down(&mut self) {
        let visible_len = self.visible_repo_count();
        if visible_len == 0 {
            return;
        }

        let i = match self.selected_index() {
            Some(i) => i.saturating_add(1).min(visible_len.saturating_sub(1)),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub(crate) fn prev_registered_tag_page(&mut self) {
        let page_count = tag_page_count(self.registered_tags().len());
        if page_count <= 1 || self.registered_tag_page == 0 {
            return;
        }

        self.registered_tag_page -= 1;
        self.status_message = format!(
            "registered tag page: {}/{}",
            self.registered_tag_page + 1,
            page_count
        );
    }

    pub(crate) fn next_registered_tag_page(&mut self) {
        let page_count = tag_page_count(self.registered_tags().len());
        if page_count <= 1 || self.registered_tag_page + 1 >= page_count {
            return;
        }

        self.registered_tag_page += 1;
        self.status_message = format!(
            "registered tag page: {}/{}",
            self.registered_tag_page + 1,
            page_count
        );
    }

    pub(crate) fn toggle_sort_mode(&mut self) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        self.sort_mode = self.sort_mode.toggle();
        self.restore_selection(selected_repo_name.as_deref());
        self.status_message = format!("sort: {}", self.sort_mode.label());
    }

    pub(crate) fn restore_selection(&mut self, repo_name: Option<&str>) {
        self.sync_selection();

        let Some(repo_name) = repo_name else {
            return;
        };

        let Some(position) = self
            .visible_repo_indices()
            .iter()
            .position(|&index| self.data.repos[index].name == repo_name)
        else {
            return;
        };

        self.list_state.select(Some(position));
    }
}
