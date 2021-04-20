use crate::request_data::RequestData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Data = Arc<RequestData>;
pub type Map = HashMap<u16, i32>;
pub type Countermap = Arc<Mutex<Map>>;
pub type Logger = Arc<slog::Logger>;
