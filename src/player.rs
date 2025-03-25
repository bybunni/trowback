use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

// Player component
#[derive(Component)]
pub struct Player;

// Player movement speed (could be a resource instead)
const MOVE_SPEED: f32 = 3.0;

// Create a player entity
pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Add player sphere
    commands.spawn((
        Player,
        Mesh3d(meshes.add(Mesh::from(bevy::prelude::Sphere { radius: 0.5 }))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}

// Handle player movement based on keyboard input
pub fn move_player(
    mut transforms: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in transforms.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Get directional input
        if keys.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
        if keys.pressed(KeyCode::KeyS) { direction.z += 1.0; }
        if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        if direction.length() > 0.0 {
            // Normalize and apply movement, factoring in delta time for smooth movement
            transform.translation += MOVE_SPEED * direction.normalize() * time.delta_secs();
            // Adjust player height based on terrain height
            let x = transform.translation.x + 20.0; // Adjust for terrain offset
            let z = transform.translation.z + 20.0; // Adjust for terrain offset
            
            // Make sure we're within the terrain bounds
            if x >= 0.0 && x <= 40.0 && z >= 0.0 && z <= 40.0 {
                // Calculate terrain height at player position using noise function
                let perlin = Perlin::new(123);
                let nx = x / 40.0 * 5.0;
                let nz = z / 40.0 * 5.0;
                let terrain_height = perlin.get([nx as f64, nz as f64]) as f32 * 1.0; // 1.0 is height scale
                
                // Update player height (0.5 is half the sphere height)
                transform.translation.y = terrain_height + 0.5;
            } else {
                // Default height if outside terrain
                transform.translation.y = 0.5;
            }
        }
    }
}

// Plugin for the player module
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player);
    }
}
