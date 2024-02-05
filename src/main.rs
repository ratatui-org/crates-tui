mod action;
mod app;
mod cli;
mod config;
mod errors;
mod logging;
mod tui;
mod widgets;

use color_eyre::eyre::Result;
use tokio::sync::mpsc;

use crate::{config::initialize_config, errors::initialize_panic_handler, logging::initialize_logging};

async fn tokio_main() -> Result<()> {
  let cli = cli::get();
  initialize_config(&cli)?;

  initialize_logging()?;
  initialize_panic_handler()?;

  if cli.print_default_config {
    println!("{}", toml::to_string_pretty(config::get())?);
    return Ok(());
  }

  let mut tui = tui::Tui::new()?.tick_rate(config::get().tick_rate).frame_rate(config::get().frame_rate);

  let (tx, rx) = mpsc::unbounded_channel();
  let mut app = app::App::new(tx);
  app.run(&mut tui, rx).await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  if let Err(e) = tokio_main().await {
    eprintln!("{} error: Something went wrong.", env!("CARGO_PKG_NAME"));
    Err(e)
  } else {
    Ok(())
  }
}
