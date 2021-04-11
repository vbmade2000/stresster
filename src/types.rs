use crate::data::Data;
use crate::enums::HTTPMethods;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type URL = Arc<String>;
pub type METHOD = Arc<HTTPMethods>;
pub type DATA = Arc<Data>;
pub type MAP = HashMap<u16, i32>;
pub type COUNTERMAP = Arc<Mutex<MAP>>;
pub type LOGGER = Arc<slog::Logger>;
