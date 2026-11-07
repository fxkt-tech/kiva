//! Content distances and speeds use the same League-to-world scale. Health,
//! damage, attack frequency and durations keep their original units.
pub const LEAGUE_UNITS_PER_WORLD: f32 = 100.0;

pub const fn world_units(league: f32) -> f32 {
    league / LEAGUE_UNITS_PER_WORLD
}

pub const fn league_units(world: f32) -> f32 {
    world * LEAGUE_UNITS_PER_WORLD
}

pub(crate) const fn regen_per_second(per_five_seconds: f32) -> f32 {
    per_five_seconds / 5.0
}
