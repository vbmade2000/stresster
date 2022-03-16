use async_trait::async_trait;

use crate::types::{Countermap, Logger};

#[async_trait()]
pub trait OutputProducer {
    /// Contains a logic to render the output.
    async fn produce(&self, counter_map: Countermap, logger: Logger) -> anyhow::Result<()>;

    /// Returns the name of the format the concrete producer type is going to produce.
    async fn format_name(&self) -> String;
}
