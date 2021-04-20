use crate::enums::HttpMethods;
use reqwest::header::HeaderMap;
use serde_json::Value;

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
            method: HttpMethods::fromstr("get".to_owned()).unwrap(),
            url: "".to_owned(),
            cert_path: "".to_owned(),
        }
    }
}
