use clap::{Parser, Subcommand as ClapSubcommand};

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "own-repos-curator")]
#[command(about = "TUI for curating descriptions of your GitHub repositories")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Subcommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ClapSubcommand)]
pub(crate) enum Subcommand {
    /// Print the build-time commit hash
    Hash,
    /// Self-update the application from GitHub
    Update,
}
