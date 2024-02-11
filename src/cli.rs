use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;
use serde_with::{serde_as, skip_serializing_none, NoneAsEmptyString};
use tracing::level_filters::LevelFilter;

pub fn version() -> String {
    let git_describe = if env!("VERGEN_GIT_DESCRIBE") != "VERGEN_IDEMPOTENT_OUTPUT" {
        format!("-{}", env!("VERGEN_GIT_DESCRIBE"))
    } else {
        "".into()
    };
    let version_message = format!(
        "{}{} ({})",
        env!("CARGO_PKG_VERSION"),
        git_describe,
        env!("VERGEN_BUILD_DATE"),
    );
    let author = clap::crate_authors!();

    format!(
        "\
{version_message}

Authors: {author}"
    )
}

/// Command line arguments.
///
/// Implements Serialize so that we can use it as a source for Figment
/// configuration.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Parser, Serialize)]
#[command(author, version = version(), about, long_about = None)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 1.0)]
    pub tick_rate: f64,

    /// Print default configuration
    #[arg(long)]
    pub print_default_config: bool,

    /// A path to a crates-tui configuration file.
    #[arg(short, long, value_name = "FILE")]
    pub config_file: Option<PathBuf>,

    /// A path to a base16 color file.
    #[arg(long, value_name = "FILE")]
    pub color_file: Option<PathBuf>,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 15.0)]
    pub frame_rate: f64,

    /// The directory to use for storing application data.
    #[arg(long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// The log level to use.
    ///
    /// Valid values are: error, warn, info, debug, trace, off. The default is
    /// info.
    #[arg(long, value_name = "LEVEL", alias = "log")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub log_level: Option<LevelFilter>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
