//! Game-independent achievement registration and deterministic progress evaluation.
//! Games own their facts and predicates; callers own transport and persistence.

mod activity;
mod facts;
mod progress;
mod registry;

pub use activity::*;
pub use facts::*;
pub use progress::*;
pub use registry::*;
