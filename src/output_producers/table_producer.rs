use crate::types::{Countermap, Logger};
use prettytable::{Cell, Row, Table};

/// Shows output in tabular format
pub async fn produce_tabular_output(counter_map: Countermap, logger: Logger) {
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
