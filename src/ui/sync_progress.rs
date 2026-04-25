use super::{layout::centered_popup_rect, theme};
use crate::app::SyncProgressState;
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

pub(super) fn render_sync_progress(f: &mut ratatui::Frame, area: Rect, state: &SyncProgressState) {
    let lines = vec![
        Line::from(Span::styled(
            state.title.clone(),
            theme::popup_title().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("状態: ".to_string(), theme::popup_muted()),
            Span::styled(state.phase.clone(), theme::popup_soft()),
        ]),
        Line::from(vec![
            Span::styled("経過: ".to_string(), theme::popup_muted()),
            Span::styled(format!("{}秒", state.elapsed_secs), theme::popup_warning()),
        ]),
    ];

    let popup_width = 64.min(area.width.saturating_sub(4).max(1));
    let popup_height = 7.min(area.height.max(1));
    let popup = centered_popup_rect(area, popup_width, popup_height);

    f.render_widget(Clear, popup);
    let block = theme::popup_block(" Sync ");
    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(Paragraph::new(lines).style(theme::popup_soft()), inner);
}

#[cfg(test)]
mod tests {
    use super::render_sync_progress;
    use crate::{app::SyncProgressState, ui::test_utils::render_overlay_text};
    use ratatui::layout::Rect;

    #[test]
    fn sync_progress_overlay_renders_phase_and_elapsed_seconds() {
        let state = SyncProgressState {
            title: "GitHub sync".to_string(),
            phase: "fetching public repos".to_string(),
            elapsed_secs: 12,
        };

        let rendered = render_overlay_text(80, 20, |f| {
            render_sync_progress(f, Rect::new(0, 0, 80, 20), &state);
        });

        assert!(rendered.contains("GitHub sync"));
        assert!(rendered.contains("fetching public repos"));
        assert!(rendered.contains("12秒"));
    }
}
