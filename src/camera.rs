use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::player::Player;
use crate::terrain::get_terrain_height;

// Component for tracking the camera that follows the player
#[derive(Component)]
pub struct FollowCamera;

// Component for the targeting cursor
#[derive(Component)]
pub struct TargetCursor;

// Resource to track mouse position and cursor target
#[derive(Resource)]
pub struct MouseLook {
    pub cursor_position: Vec2,
    pub target_position: Vec3,
    pub is_initialized: bool,
}

// Setup the camera and targeting cursor
pub fn spawn_camera(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) {
    // Spawn the camera
    commands.spawn((
        Camera3d::default(),
        FollowCamera,
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Create a simple targeting cursor (small red sphere)
    commands.spawn((
        TargetCursor,
        Mesh3d(meshes.add(Sphere::new(0.2).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.0, 0.0, 0.7),
            emissive: Color::srgba(1.0, 0.2, 0.2, 1.0).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Visibility::Hidden,
    ));
    
    // Initialize the MouseLook resource
    commands.insert_resource(MouseLook {
        cursor_position: Vec2::ZERO,
        target_position: Vec3::ZERO,
        is_initialized: false,
    });
}

// Split the camera handling into separate systems to avoid borrow checker issues

// System to update cursor position from mouse input
pub fn update_mouse_position(
    mut mouse_look: ResMut<MouseLook>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(window) = window_query.get_single().ok() {
        if let Some(cursor_position) = window.cursor_position() {
            // Update the cursor position in our resource
            mouse_look.cursor_position = cursor_position;
        }
    }
}

// System to handle cursor raycasting and positioning
pub fn cursor_raycasting(
    // Remove unused player_query
    camera_query: Query<(&Camera, &GlobalTransform), With<FollowCamera>>,
    mut cursor_query: Query<(&mut Transform, &mut Visibility), With<TargetCursor>>,
    mut mouse_look: ResMut<MouseLook>
) {
    // Exit early if needed components aren't available
    if let (Ok((camera, camera_transform)), Some(cursor_position)) = (
        camera_query.get_single(),
        if mouse_look.cursor_position != Vec2::ZERO { Some(mouse_look.cursor_position) } else { None }
    ) {
        // Cast a ray from the cursor position into the 3D world
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Calculate the point where the ray intersects the terrain
            let mut hit_position = Vec3::ZERO;
            let mut hit_found = false;
            
            // We'll check multiple points along the ray to find where it hits the terrain
            let ray_start = ray.origin + ray.direction * 5.0;
            
            // Sample multiple points along the ray
            for i in 0..20 {
                let distance = i as f32 * 2.0;
                let sample_pos = ray_start + ray.direction * distance;
                let terrain_height = get_terrain_height(sample_pos.x, sample_pos.z);
                
                // Check if this sample is at or below the terrain height
                if sample_pos.y <= terrain_height {
                    hit_position = Vec3::new(sample_pos.x, terrain_height, sample_pos.z);
                    hit_found = true;
                    break;
                }
            }
            
            // If we found a hit, update the cursor position
            if hit_found {
                mouse_look.target_position = hit_position;
                mouse_look.is_initialized = true;
                
                // Update the cursor mesh
                if let Ok((mut cursor_transform, mut visibility)) = cursor_query.get_single_mut() {
                    cursor_transform.translation = hit_position + Vec3::new(0.0, 0.1, 0.0); // Slightly above terrain
                    *visibility = Visibility::Visible;
                }
            }
        }
    }
}

// System to update camera position based on player and cursor
pub fn update_camera_position(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<Player>)>,
    mouse_look: Res<MouseLook>,
    time: Res<Time>,
) {
    // Exit early if player or camera isn't available
    if let (Ok(player_transform), Ok(mut camera_transform)) = (
        player_query.get_single(),
        camera_query.get_single_mut()
    ) {
        // Calculate a dynamic camera offset that maintains player view but angles toward cursor
        let base_offset = Vec3::new(-3.0, 3.5, 6.0);
        
        // Calculate the desired camera position (behind and above the player)
        let target_position = player_transform.translation + base_offset;
        
        // Smoothly interpolate the camera position
        let smoothness = 5.0;
        camera_transform.translation = camera_transform.translation.lerp(
            target_position, 
            smoothness * time.delta_secs()
        );
        
        // Make camera look at player or cursor based on mouse state
        if mouse_look.is_initialized {
            // Calculate a blended look target between player and cursor
            // This keeps the player in view while angling toward the cursor
            let player_pos = player_transform.translation + Vec3::new(0.0, 0.5, 0.0);
            let cursor_weight = 0.6; // Adjust this to change how much the camera focuses on cursor vs player
            let look_target = player_pos.lerp(mouse_look.target_position, cursor_weight);
            
            // Smoothly rotate the camera to look at the target
            let target_rotation = Transform::from_translation(camera_transform.translation)
                .looking_at(look_target, Vec3::Y).rotation;
            camera_transform.rotation = camera_transform.rotation.slerp(target_rotation, 8.0 * time.delta_secs());
        } else {
            // Default to looking at player if mouse not initialized
            let look_target = player_transform.translation + Vec3::new(0.0, 0.5, 0.0);
            camera_transform.look_at(look_target, Vec3::Y);
        }
    }
}

// Plugin for the camera module
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Add systems in a specific order and ensure they don't conflict on component access
        app
            // First update the mouse position (just tracks mouse movement)
            .add_systems(Update, update_mouse_position)
            // Then handle cursor raycasting in a separate system group to avoid conflicts
            .add_systems(Update, cursor_raycasting.after(update_mouse_position))
            // Finally update camera position
            .add_systems(Update, update_camera_position.after(cursor_raycasting));
    }
}
