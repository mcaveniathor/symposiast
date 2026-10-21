use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, profile_work)
        .run();
}

fn profile_work() {
    let _span = info_span!("example_work", name = "example_work").entered();
    let total: u64 = (0..1_000_u64)
        .map(std::hint::black_box)
        .sum();
    std::hint::black_box(total);
}
