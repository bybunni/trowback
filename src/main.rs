use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, camera_follow))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FollowCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    
    // Setup camera
    commands.spawn((
        Camera3d::default(),
        FollowCamera,
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add player sphere
    commands.spawn((
        Player,
        Mesh3d(meshes.add(Mesh::from(bevy::prelude::Sphere { radius: 0.5 }))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Add ground plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(bevy::prelude::Plane3d::new(Vec3::Y, Vec2::splat(5.0))))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
    ));
}

const MOVE_SPEED: f32 = 6.0;

fn move_player(
    mut transforms: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in transforms.iter_mut() {
        let mut direction = Vec3::ZERO;
        // Forward/backward movement (in Z axis for 3D)
        if keys.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        // Left/right movement (in X axis)
        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            // Normalize and apply movement, factoring in delta time for smooth movement
            transform.translation += MOVE_SPEED * direction.normalize() * time.delta_secs();
            // Keep the player on the ground
            transform.translation.y = 0.5; // Half the height of the sphere
        }
    }
}

// Camera follows the player with a slight offset
fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    // If we have a player and a camera
    if let (Ok(player_transform), Ok(mut camera_transform)) = (player_query.get_single(), camera_query.get_single_mut()) {
        // Calculate the desired camera position (behind and above the player)
        let offset = Vec3::new(-2.0, 2.5, 5.0);
        let target_position = player_transform.translation + offset;
        
        // Smoothly interpolate the camera position
        let smoothness = 5.0;
        camera_transform.translation = camera_transform.translation.lerp(
            target_position, 
            smoothness * time.delta_secs()
        );
        
        // Make camera look at player
        let look_target = player_transform.translation + Vec3::new(0.0, 0.5, 0.0);
        camera_transform.look_at(look_target, Vec3::Y);
    }
}