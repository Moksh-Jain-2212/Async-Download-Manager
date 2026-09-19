use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Download {
        url: String,
    },

    List,

    Pause {
        id: u64,
    },

    Resume {
        id: u64,
    },

    Cancel {
        id: u64,
    },
}