use super::theme;
use crate::model::DEFAULT_GROUP_NAME;
use ratatui::{style::Style, text::Span};

pub(super) fn style_for_group(registered_groups: &[String], group: &str) -> Style {
    if group == DEFAULT_GROUP_NAME {
        return theme::monokai_light_gray();
    }

    registered_groups
        .iter()
        .filter(|registered_group| registered_group.as_str() != DEFAULT_GROUP_NAME)
        .position(|registered_group| registered_group == group)
        .map(theme::monokai_tag)
        .unwrap_or_else(theme::monokai_light_gray)
}

pub(super) fn span_for_group(registered_groups: &[String], group: &str) -> Span<'static> {
    Span::styled(group.to_string(), style_for_group(registered_groups, group))
}

#[cfg(test)]
mod tests {
    use super::{span_for_group, style_for_group};
    use crate::{model::DEFAULT_GROUP_NAME, ui::theme};

    #[test]
    fn default_group_uses_light_gray() {
        let registered_groups = vec![
            "apps".to_string(),
            DEFAULT_GROUP_NAME.to_string(),
            "tools".to_string(),
        ];

        assert_eq!(
            style_for_group(&registered_groups, DEFAULT_GROUP_NAME),
            theme::monokai_light_gray()
        );
    }

    #[test]
    fn non_default_groups_use_monokai_colors_in_registered_order() {
        let registered_groups = vec![
            "apps".to_string(),
            DEFAULT_GROUP_NAME.to_string(),
            "tools".to_string(),
            "web".to_string(),
        ];

        assert_eq!(
            style_for_group(&registered_groups, "apps"),
            theme::monokai_tag(0)
        );
        assert_eq!(
            style_for_group(&registered_groups, "tools"),
            theme::monokai_tag(1)
        );
        assert_eq!(
            span_for_group(&registered_groups, "web").style,
            theme::monokai_tag(2)
        );
    }
}
