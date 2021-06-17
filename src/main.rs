extern crate clap;
#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod enums;
mod helper;
pub mod output_producers;
mod request_data;
mod stresster;
mod types;

use stresster::Stresster;

const LOG_PATH: &str = "stresster.log";

#[tokio::main]
async fn main() {
    Stresster {
        log_path: LOG_PATH.to_owned(),
    }
    .run()
    .await;
}
