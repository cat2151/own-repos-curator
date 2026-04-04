use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{block::Title, Block, Borders},
};

pub(super) const BACKGROUND: Color = Color::Rgb(39, 40, 34);
pub(super) const SURFACE: Color = Color::Rgb(49, 51, 43);
pub(super) const SELECTION: Color = Color::Rgb(73, 72, 62);
pub(super) const FOREGROUND: Color = Color::Rgb(248, 248, 242);
pub(super) const LIGHT_GRAY: Color = Color::Rgb(221, 218, 205);
pub(super) const COMMENT: Color = Color::Rgb(117, 113, 94);
pub(super) const CYAN: Color = Color::Rgb(102, 217, 239);
pub(super) const YELLOW: Color = Color::Rgb(230, 219, 116);
pub(super) const ORANGE: Color = Color::Rgb(253, 151, 31);
pub(super) const RED: Color = Color::Rgb(249, 38, 114);
pub(super) const GREEN: Color = Color::Rgb(166, 226, 46);
pub(super) const PURPLE: Color = Color::Rgb(174, 129, 255);

const MONOKAI_TAG_COLORS: [Color; 6] = [RED, ORANGE, YELLOW, GREEN, CYAN, PURPLE];

fn style(bg: Color, fg: Color) -> Style {
    Style::default().bg(bg).fg(fg)
}

pub(super) fn screen() -> Style {
    style(BACKGROUND, FOREGROUND)
}

pub(super) fn body() -> Style {
    style(BACKGROUND, FOREGROUND)
}

pub(super) fn soft() -> Style {
    style(BACKGROUND, LIGHT_GRAY)
}

pub(super) fn muted() -> Style {
    style(BACKGROUND, COMMENT)
}

pub(super) fn accent() -> Style {
    style(BACKGROUND, CYAN)
}

pub(super) fn warning() -> Style {
    style(BACKGROUND, YELLOW)
}

pub(super) fn monokai_tag(index: usize) -> Style {
    Style::default().fg(MONOKAI_TAG_COLORS[index % MONOKAI_TAG_COLORS.len()])
}

pub(super) fn panel_title() -> Style {
    style(BACKGROUND, ORANGE).add_modifier(Modifier::BOLD)
}

pub(super) fn highlight() -> Style {
    Style::default().bg(SELECTION).add_modifier(Modifier::BOLD)
}

pub(super) fn popup() -> Style {
    style(SURFACE, FOREGROUND)
}

pub(super) fn popup_soft() -> Style {
    style(SURFACE, LIGHT_GRAY)
}

pub(super) fn popup_muted() -> Style {
    style(SURFACE, COMMENT)
}

pub(super) fn popup_warning() -> Style {
    style(SURFACE, YELLOW)
}

pub(super) fn popup_title() -> Style {
    style(SURFACE, ORANGE).add_modifier(Modifier::BOLD)
}

pub(super) fn popup_highlight() -> Style {
    Style::default().bg(SELECTION).add_modifier(Modifier::BOLD)
}

pub(super) fn panel_block<'a, T>(title: T) -> Block<'a>
where
    T: Into<Title<'a>>,
{
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(screen())
        .border_style(muted())
        .title_style(panel_title())
}

pub(super) fn popup_block<'a, T>(title: T) -> Block<'a>
where
    T: Into<Title<'a>>,
{
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(popup())
        .border_style(popup_muted())
        .title_style(popup_title())
}

#[cfg(test)]
mod tests {
    use super::{highlight, popup_highlight, SELECTION};
    use ratatui::style::Modifier;

    #[test]
    fn highlight_preserves_item_foreground_colors() {
        let style = highlight();

        assert_eq!(style.bg, Some(SELECTION));
        assert_eq!(style.fg, None);
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn popup_highlight_only_adds_selection_background() {
        let style = popup_highlight();

        assert_eq!(style.bg, Some(SELECTION));
        assert_eq!(style.fg, None);
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }
}
