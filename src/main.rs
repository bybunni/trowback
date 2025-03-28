use bevy::prelude::*;

// Import our modules
mod player;
mod camera;
mod terrain;
mod assets;
mod projectile;

// Import specific items we need
use player::{PlayerPlugin, spawn_player};
use camera::{CameraPlugin, spawn_camera};
use terrain::TerrainPlugin;
use projectile::ProjectilePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add our custom plugins
        .add_plugins((PlayerPlugin, CameraPlugin, TerrainPlugin, ProjectilePlugin))
        .add_systems(Startup, setup)
        .run();
}

// Setup function for initializing the game world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // Add a light source
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
    
    // Add camera using the camera module
    spawn_camera(&mut commands, &mut meshes, &mut materials);

    // Add player using the player module
    spawn_player(&mut commands, &mut meshes, &mut materials, &mut textures);

    // Terrain is now managed by the TerrainPlugin with dynamic chunk loading
}