use reqwest::header::HeaderMap;
use serde_json::Value;
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
}

impl From<String> for OutputFormat {
    fn from(output_format: String) -> Self {
        if output_format.to_lowercase() == "json" {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        }
    }
}

impl From<&str> for OutputFormat {
    fn from(output_format: &str) -> Self {
        if output_format.to_lowercase() == "json" {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Increment(u16),
    Exit,
}

#[derive(Debug, Clone)]
pub enum HttpMethods {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethods {
    pub fn fromstr(method: &str) -> Option<HttpMethods> {
        match method {
            "get" => Some(HttpMethods::Get),
            "post" => Some(HttpMethods::Post),
            "put" => Some(HttpMethods::Put),
            "delete" => Some(HttpMethods::Delete),
            "patch" => Some(HttpMethods::Patch),
            _ => None,
        }
    }
}

/// Struct to hold data related to request like payload, header etc
#[derive(Debug, Clone)]
pub struct RequestData {
    /// Actual JSON payload to be supplied
    pub payload: Value,

    /// HTTP headers to be supplied
    pub headers: HeaderMap,

    /// HTTP Method/Verb to use
    pub method: HttpMethods,

    /// URL
    pub url: String,

    /// SSL certificate path
    pub cert_path: String,
}

impl Default for RequestData {
    /// Create and return instance of ResuestData struct with default values
    fn default() -> RequestData {
        RequestData {
            payload: serde_json::from_str("{}").unwrap(),
            headers: HeaderMap::new(),
            method: HttpMethods::fromstr("get").unwrap(),
            url: "".to_owned(),
            cert_path: "".to_owned(),
        }
    }
}
