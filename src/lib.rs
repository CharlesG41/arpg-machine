//! Simulation for the ARPG Machine prototype.
//!
//! The simulation never draws anything: it only knows positions, stats and
//! identifiers, and reports what happened through [`world::Event`]s. The
//! renderer in the binary decides what everything looks like, so the visuals
//! can be replaced without touching the rules.

pub mod content;
pub mod grid;
pub mod math;
pub mod rng;
pub mod world;
