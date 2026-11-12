//! A bamboo-grove action combat prototype: headless simulation rules, level
//! assembly and a presentation client, depending in that one direction.
mod client;
pub mod gameplay;
pub mod simulation;

pub fn run() {
    client::run();
}
