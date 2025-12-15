//! Block subscription and epoch tracking for Bittensor
//!
//! This module provides:
//! - Block subscription via `subscribe_finalized_blocks`
//! - Epoch tracking and phase detection (evaluation, commit, reveal)
//! - Events for epoch transitions

mod epoch_tracker;
mod listener;

pub use epoch_tracker::*;
pub use listener::*;
