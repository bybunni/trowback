use bevy::{
    prelude::*,
    sprite::MeshMaterial2d,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player;

// Spawn an entity with a mesh using `ColorMaterial`.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d::default());
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE))),
    ));
}

const MOVE_SPEED: f32 = 6.0;

fn move_player(
    mut transforms: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for mut transform in transforms.iter_mut() {
        let mut direction = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if 0.0 < direction.length() {
            transform.translation += MOVE_SPEED * direction.normalize();
        }
    }
}