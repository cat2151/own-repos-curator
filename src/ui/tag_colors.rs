use super::theme;
use ratatui::{style::Style, text::Span};

pub(super) fn style_for_tag(registered_tags: &[String], tag: &str) -> Style {
    registered_tags
        .iter()
        .position(|registered_tag| registered_tag == tag)
        .map(theme::monokai_tag)
        .unwrap_or_else(theme::soft)
}

pub(super) fn span_for_tag(registered_tags: &[String], tag: &str) -> Span<'static> {
    Span::styled(tag.to_string(), style_for_tag(registered_tags, tag))
}

#[cfg(test)]
mod tests {
    use super::{span_for_tag, style_for_tag};
    use crate::ui::theme;

    #[test]
    fn style_for_tag_uses_registered_order() {
        let registered_tags = vec!["rust".to_string(), "zig".to_string(), "go".to_string()];

        assert_eq!(
            style_for_tag(&registered_tags, "zig"),
            theme::monokai_tag(1)
        );
    }

    #[test]
    fn unregistered_tags_fall_back_to_soft_style() {
        let registered_tags = vec!["rust".to_string()];

        assert_eq!(style_for_tag(&registered_tags, "unknown"), theme::soft());
        assert_eq!(
            span_for_tag(&registered_tags, "unknown").style,
            theme::soft()
        );
    }
}
