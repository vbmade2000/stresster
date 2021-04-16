use crate::enums::HTTPMethods;
use reqwest::header::HeaderMap;
use serde_json::Value;

/// Struct to hold data related to request like payload, header etc
#[derive(Debug, Clone)]
pub struct Data {
    /// Actual JSON payload to be supplied
    pub payload: Value,

    /// HTTP headers to be supplied
    pub headers: HeaderMap,

    /// HTTP Method/Verb to use
    pub method: HTTPMethods,

    /// URL
    pub url: String,

    /// SSL certificate path
    pub cert_path: String,
}

impl Default for Data {
    /// Create and return instance of Data struct with default values
    fn default() -> Data {
        Data {
            payload: serde_json::from_str("{}").unwrap(),
            headers: HeaderMap::new(),
            method: HTTPMethods::fromstr("GET".to_string()).unwrap(),
            url: "".to_string(),
            cert_path: "".to_string(),
        }
    }
}
