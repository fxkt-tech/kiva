//! Kiva's reusable headless simulation and desktop/browser application.
mod client;
pub mod simulation;

pub fn run() {
    client::run();
}
