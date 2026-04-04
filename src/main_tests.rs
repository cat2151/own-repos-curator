use super::*;
use clap::Parser;

#[test]
fn parse_subcommand_recognizes_hash() {
    let cli = Cli::try_parse_from(["repocurator", "hash"]).unwrap();
    assert_eq!(cli.command, Some(Subcommand::Hash));
}

#[test]
fn parse_subcommand_recognizes_update() {
    let cli = Cli::try_parse_from(["repocurator", "update"]).unwrap();
    assert_eq!(cli.command, Some(Subcommand::Update));
}

#[test]
fn parse_subcommand_is_optional() {
    let cli = Cli::try_parse_from(["repocurator"]).unwrap();
    assert_eq!(cli.command, None);
}

#[test]
fn parse_subcommand_rejects_unknown_command() {
    assert!(Cli::try_parse_from(["repocurator", "unknown"]).is_err());
}
