extern crate clap;
#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod helper;
pub mod output_producers;
mod stresster;
mod types;

use stresster::Stresster;

const LOG_PATH: &str = "stresster.log";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Stresster {
        log_path: LOG_PATH.to_owned(),
    }
    .run()
    .await?;
    Ok(())
}
