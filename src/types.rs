use crate::request_data::RequestData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Data = Arc<RequestData>;
pub type Map = HashMap<u16, i32>;
pub type Countermap = Arc<Mutex<Map>>;
pub type Logger = Arc<slog::Logger>;

#[derive(PartialEq)]
pub enum OutputFormat {
    Json,
    Table,
    None,
}

impl From<String> for OutputFormat {
    fn from(output_format: String) -> Self {
        if output_format.to_lowercase() == "json" {
            OutputFormat::Json
        } else if output_format.to_lowercase() == "table" {
            OutputFormat::Table
        } else {
            OutputFormat::None
        }
    }
}

impl From<&str> for OutputFormat {
    fn from(output_format: &str) -> Self {
        if output_format.to_lowercase() == "json" {
            OutputFormat::Json
        } else if output_format.to_lowercase() == "table" {
            OutputFormat::Table
        } else {
            OutputFormat::None
        }
    }
}
