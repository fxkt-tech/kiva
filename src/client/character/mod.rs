//! Skeletal character presentation, independent of simulation and input rules.
mod animation;
mod model;
mod poses;
mod rig;

pub(super) use animation::{RigPose, record_events, sync_attachments, sync_poses};
pub(super) use model::{ModelAssets, setup, spawn_model};
pub(super) use rig::AnimationLibrary;

#[cfg(test)]
mod tests;
