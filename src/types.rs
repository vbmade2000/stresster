use crate::enums::HTTPMethods;
use serde_json::Value;
use slog;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type URL = Arc<String>;
pub type METHOD = Arc<HTTPMethods>;
pub type PAYLOAD = Arc<Value>;
pub type MAP = HashMap<u16, i32>;
pub type COUNTERMAP = Arc<Mutex<MAP>>;
pub type LOGGER = Arc<slog::Logger>;
