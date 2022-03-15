use async_trait::async_trait;

use crate::output_producers::output_producer;
use crate::types::{Countermap, Logger};

/// Struct that produces JSON output
pub struct JSONProducer;

#[async_trait()]
impl output_producer::OutputProducer for JSONProducer {
    async fn produce(&self, counter_map: Countermap, logger: Logger) {
        let logger = logger.clone();
        debug!(logger, "Writing output in JSON format");
        let map = counter_map.lock().await;

        // Create nice JSON using serde
        let serialized_json = serde_json::to_string_pretty(&*map);
        println!("{}", serialized_json.unwrap());
    }

    async fn format_name(&self) -> String {
        "Json".to_string()
    }
}
