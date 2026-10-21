use bevy::prelude::*;

#[derive(Message)]
struct EnemyDied(u32);

#[derive(Resource)]
struct Score(u32);

fn update_score(mut messages: MessageReader<EnemyDied>, mut score: ResMut<Score>) {
    for message in messages.read() {
        score.0 += message.0;
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Score(0))
        .add_message::<EnemyDied>()
        .add_systems(Update, update_score);

    app.world_mut()
        .resource_mut::<Messages<EnemyDied>>()
        .write(EnemyDied(3));

    app.update();

    assert_eq!(app.world().resource::<Score>().0, 3);
}
