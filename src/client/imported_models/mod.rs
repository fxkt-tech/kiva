//! Authored model assets adapt simulation facts to paused, explicitly sampled clips.
mod catalog;
mod runtime;

pub(super) use catalog::{ability_icon, portrait};
pub(super) use runtime::{ImportedModelView, ImportedModelsPlugin};

#[cfg(test)]
mod tests;
