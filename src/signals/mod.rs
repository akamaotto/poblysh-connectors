//! # Signals Module
//!
//! This module contains the signal processing pipeline including the weak signal engine
//! that processes normalized signals and promotes them to grounded signals.

pub mod weak_engine;

pub use weak_engine::{WeakSignalEngine, WeakSignalEngineConfig};
