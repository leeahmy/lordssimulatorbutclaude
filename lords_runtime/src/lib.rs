//! Runs a [`lords_sim::Simulation`].
//!
//! [`Runtime::spawn`] moves a simulation to a thread of its own, which ticks it at the pace
//! of its [`Speed`].

mod clock;
mod engine;
mod runtime;
mod snapshot;
mod speed;

pub use runtime::Runtime;
pub use snapshot::Snapshot;
pub use speed::Speed;
