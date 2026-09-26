//! Native-only Calendar Core entry point. Provider connectors normalize their payloads
//! into `ExternalEventInput` and call the repository reconciliation API.
pub use crate::db::repositories::external_events::{
    reconcile_in_transaction, ExternalEventInput, ReconciliationMode,
};
