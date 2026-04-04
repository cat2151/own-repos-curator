use super::*;

#[test]
fn parse_subcommand_recognizes_hash() {
    let args = vec!["repocurator".to_string(), "hash".to_string()];
    assert_eq!(parse_subcommand(&args), Some(Subcommand::Hash));
}

#[test]
fn parse_subcommand_recognizes_update() {
    let args = vec!["repocurator".to_string(), "update".to_string()];
    assert_eq!(parse_subcommand(&args), Some(Subcommand::Update));
}

#[test]
fn parse_subcommand_ignores_unknown_command() {
    let args = vec!["repocurator".to_string(), "unknown".to_string()];
    assert_eq!(parse_subcommand(&args), None);
}
