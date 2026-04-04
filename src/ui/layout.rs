use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
};

pub(super) fn centered_popup_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_height = height.clamp(1, area.height.max(1));
    let popup_width = width.clamp(1, area.width.max(1));
    let popup = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area)[0];
    Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(popup)[0]
}

pub(super) fn popup_width_for_lines<'a>(
    lines: &[Line<'a>],
    area: Rect,
    min_width: u16,
    max_width: u16,
) -> u16 {
    let available_width = area.width.saturating_sub(4).max(1);
    let min_width = min_width.min(available_width);
    let max_width = max_width.min(available_width).max(min_width);
    let content_width = lines.iter().map(Line::width).max().unwrap_or(0) as u16;
    (content_width + 4).clamp(min_width, max_width)
}

pub(super) fn popup_height_for_lines<'a>(
    lines: &[Line<'a>],
    popup_width: u16,
    area: Rect,
    min_height: u16,
) -> u16 {
    let inner_width = popup_width.saturating_sub(2).max(1) as usize;
    let wrapped_line_count = lines
        .iter()
        .map(|line| {
            let width = line.width();
            width.max(1).div_ceil(inner_width)
        })
        .sum::<usize>() as u16;
    let available_height = area.height.saturating_sub(4).max(1);
    let min_height = min_height.min(available_height);
    (wrapped_line_count + 2).clamp(min_height, available_height)
}

pub(super) fn log_pane_height(total_height: u16, expanded: bool) -> u16 {
    let available = total_height.saturating_sub(1);
    if expanded {
        (total_height / 2).max(1).min(available)
    } else {
        3.min(available)
    }
}

pub(super) fn left_pane_width(total_width: u16, desired_width: u16) -> u16 {
    if total_width <= 1 {
        return total_width;
    }

    let min_width = total_width.div_ceil(4).max(1);
    let max_width = ((u32::from(total_width) * 3) / 4) as u16;
    let max_width = max_width.min(total_width.saturating_sub(1)).max(min_width);
    desired_width.clamp(min_width, max_width)
}

#[cfg(test)]
mod tests {
    use super::{left_pane_width, log_pane_height};

    #[test]
    fn log_pane_height_is_three_lines_when_collapsed() {
        assert_eq!(log_pane_height(30, false), 3);
        assert_eq!(log_pane_height(4, false), 3);
        assert_eq!(log_pane_height(3, false), 2);
        assert_eq!(log_pane_height(1, false), 0);
    }

    #[test]
    fn log_pane_height_uses_bottom_half_when_expanded() {
        assert_eq!(log_pane_height(20, true), 10);
        assert_eq!(log_pane_height(3, true), 1);
    }

    #[test]
    fn left_pane_width_is_clamped_between_twenty_five_and_seventy_five_percent() {
        assert_eq!(left_pane_width(100, 10), 25);
        assert_eq!(left_pane_width(100, 50), 50);
        assert_eq!(left_pane_width(100, 90), 75);
    }
}
