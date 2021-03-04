use crate::enums::HTTPMethods;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type URL = Arc<String>;
pub type METHOD = Arc<HTTPMethods>;
pub type PAYLOAD = Arc<Value>;
pub type COUNTERMAP = Arc<Mutex<HashMap<u16, i32>>>;
