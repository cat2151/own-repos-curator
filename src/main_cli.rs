#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subcommand {
    Hash,
    Update,
}

pub(crate) fn parse_subcommand(args: &[String]) -> Option<Subcommand> {
    match args.get(1).map(String::as_str) {
        Some("hash") => Some(Subcommand::Hash),
        Some("update") => Some(Subcommand::Update),
        _ => None,
    }
}
