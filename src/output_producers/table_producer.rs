use crate::types::{Countermap, Logger};
use async_trait::async_trait;
use prettytable::{Cell, Row, Table};

use super::output_producer::OutputProducer;

pub struct TableProducer;

#[async_trait()]
impl OutputProducer for TableProducer {
    async fn produce(&self, counter_map: Countermap, logger: Logger) {
        let logger = logger.clone();
        debug!(logger, "Writing output in tabular format");
        // Create nice tabular view to make output easily understandable
        let mut table = Table::new();
        table.add_row(row!["Status Code", "Count"]);

        let map = counter_map.lock().await;
        for key in &*map {
            table.add_row(Row::new(vec![
                Cell::new(&key.0.to_string()),
                Cell::new(&key.1.to_string()),
            ]));
        }
        table.printstd();
    }

    async fn format_name(&self) -> String {
        "Table".to_string()
    }
}
