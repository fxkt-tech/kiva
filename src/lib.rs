//! A top-down movement demo with independent motion, level and client layers.
mod client;
pub mod gameplay;
pub mod simulation;

pub fn run() {
    client::run();
}
