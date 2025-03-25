use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
// Import the get_terrain_height function from the terrain module
use crate::terrain::get_terrain_height;

// Player component
#[derive(Component)]
pub struct Player;

// Generate a simple procedural texture for the sphere
// This creates a texture with visible patterns to make rotation apparent
fn create_sphere_texture() -> Image {
    let size = 256; // Texture size
    let mut rgba = vec![0; size * size * 4];
    
    for y in 0..size {
        for x in 0..size {
            let i = (y * size + x) * 4;
            
            // Calculate normalized coordinates from center
            let nx = (x as f32 / size as f32) * 2.0 - 1.0;
            let ny = (y as f32 / size as f32) * 2.0 - 1.0;
            
            // Skip pixels outside the circle
            if nx*nx + ny*ny > 1.0 {
                // Transparent background
                rgba[i] = 255;     // R
                rgba[i + 1] = 255; // G
                rgba[i + 2] = 255; // B
                rgba[i + 3] = 0;   // A (transparent)
                continue;
            }
            
            // Create a pattern of segments like a beach ball or billiard ball
            let angle = ny.atan2(nx);
            let segments = 8;
            let segment_id = ((angle / std::f32::consts::PI * segments as f32 / 2.0) + segments as f32) as usize % segments;
            
            // Alternating colors for segments
            match segment_id {
                0 => {
                    rgba[i] = 200;     // R
                    rgba[i + 1] = 50;  // G
                    rgba[i + 2] = 50;  // B
                    rgba[i + 3] = 255; // A
                }
                1 => {
                    rgba[i] = 50;      // R
                    rgba[i + 1] = 50;  // G
                    rgba[i + 2] = 200; // B
                    rgba[i + 3] = 255; // A
                }
                2 => {
                    rgba[i] = 200;     // R
                    rgba[i + 1] = 200; // G
                    rgba[i + 2] = 50;  // B
                    rgba[i + 3] = 255; // A
                }
                3 => {
                    rgba[i] = 50;      // R
                    rgba[i + 1] = 200; // G
                    rgba[i + 2] = 50;  // B
                    rgba[i + 3] = 255; // A
                }
                4 => {
                    rgba[i] = 200;     // R
                    rgba[i + 1] = 50;  // G
                    rgba[i + 2] = 200; // B
                    rgba[i + 3] = 255; // A
                }
                5 => {
                    rgba[i] = 200;     // R
                    rgba[i + 1] = 120; // G
                    rgba[i + 2] = 50;  // B
                    rgba[i + 3] = 255; // A
                }
                6 => {
                    rgba[i] = 230;     // R
                    rgba[i + 1] = 230; // G
                    rgba[i + 2] = 230; // B
                    rgba[i + 3] = 255; // A
                }
                _ => {
                    rgba[i] = 40;      // R
                    rgba[i + 1] = 40;  // G
                    rgba[i + 2] = 40;  // B
                    rgba[i + 3] = 255; // A
                }
            }
            
            // Add a circle pattern in the middle of each segment
            let segment_angle = angle - (segment_id as f32 * std::f32::consts::PI / (segments as f32 / 2.0));
            let segment_center_x = 0.6 * nx.signum() * segment_angle.cos();
            let segment_center_y = 0.6 * ny.signum() * segment_angle.sin();
            let dist_to_center = ((nx - segment_center_x).powi(2) + (ny - segment_center_y).powi(2)).sqrt();
            
            if dist_to_center < 0.2 {
                // Create a darker circle in each segment
                rgba[i] = rgba[i] / 2;
                rgba[i + 1] = rgba[i + 1] / 2;
                rgba[i + 2] = rgba[i + 2] / 2;
            }
        }
    }
    
    // Create the image
    let image = Image::new_fill(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &rgba,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    
    // Return the image
    image
}

// Physics component for the player
#[derive(Component)]
pub struct PlayerPhysics {
    // Velocity in world space
    pub velocity: Vec3,
    // Angular velocity (rotation around axes)
    pub angular_velocity: Vec3,
    // Mass of the player sphere (kg)
    pub mass: f32,
    // Is the player grounded?
    pub grounded: bool,
    // Momentum - preserves movement feel
    pub momentum: Vec3,
    // Previous position - used for calculating proper rotation
    pub prev_position: Vec3,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.2, // Increased from 0.8 for better stability
            grounded: false,
            momentum: Vec3::ZERO,
            prev_position: Vec3::ZERO,
        }
    }
}

// Player physics constants
const MOVE_SPEED: f32 = 1.5; // Reduced from 3.0
const GRAVITY: f32 = 9.8;
const FRICTION: f32 = 0.95; // Slightly increased friction (was 0.98)
const TERRAIN_SENSITIVITY: f32 = 0.3; // Reduced from 0.7
const MOMENTUM_FACTOR: f32 = 0.85; // Reduced from 0.92 (less momentum preservation)
const RESTITUTION: f32 = 0.4; // Reduced from 0.6 (less bouncy)
const MASS_FACTOR: f32 = 0.8; // Increased from 0.5 (feels heavier)
const MAX_SPEED: f32 = 6.0; // Reduced from 10.0

// Create a player entity
pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    texture_assets: &mut ResMut<Assets<Image>>,
) {
    // Calculate initial terrain height at spawn position
    let initial_x = 0.0;
    let initial_z = 0.0;
    let terrain_height = get_terrain_height(initial_x, initial_z);
    
    // Add player sphere positioned at the correct height on the terrain
    let initial_position = Vec3::new(initial_x, terrain_height + 0.5, initial_z);
    
    // Create a textured material for the sphere with a pattern to show rotation
    let texture_handle = texture_assets.add(create_sphere_texture());
    let material = StandardMaterial {
        base_color_texture: Some(texture_handle),
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    
    commands.spawn((
        Player,
        PlayerPhysics {
            prev_position: initial_position,
            ..Default::default()
        },
        Mesh3d(meshes.add(Mesh::from(bevy::prelude::Sphere { radius: 0.5 }))),
        MeshMaterial3d(materials.add(material)),
        Transform::from_xyz(initial_position.x, initial_position.y, initial_position.z),
    ));
}

// Handle player movement based on keyboard input and physics
pub fn move_player(
    mut player_query: Query<(&mut Transform, &mut PlayerPhysics), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    for (mut transform, mut physics) in player_query.iter_mut() {
        // Store previous position for calculating rotation
        physics.prev_position = transform.translation;
        
        let mut input_direction = Vec3::ZERO;

        // Get directional input
        if keys.pressed(KeyCode::KeyW) { input_direction.z -= 1.0; }
        if keys.pressed(KeyCode::KeyS) { input_direction.z += 1.0; }
        if keys.pressed(KeyCode::KeyA) { input_direction.x -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { input_direction.x += 1.0; }

        // Normalize input if there is any
        if input_direction.length_squared() > 0.0 {
            input_direction = input_direction.normalize();
        }
        
        // Get current terrain height and surrounding terrain heights to calculate slope
        let pos = transform.translation;
        let current_height = get_terrain_height(pos.x, pos.z);
        
        // Sample terrain at nearby points to calculate slope
        let sample_dist = 0.5;
        let height_x_pos = get_terrain_height(pos.x + sample_dist, pos.z);
        let height_x_neg = get_terrain_height(pos.x - sample_dist, pos.z);
        let height_z_pos = get_terrain_height(pos.x, pos.z + sample_dist);
        let height_z_neg = get_terrain_height(pos.x, pos.z - sample_dist);
        
        // Calculate terrain gradient (slope direction)
        let gradient = Vec3::new(
            (height_x_neg - height_x_pos) / (2.0 * sample_dist), // negative X gradient
            0.0,
            (height_z_neg - height_z_pos) / (2.0 * sample_dist)  // negative Z gradient
        );
        
        // Calculate gradient strength - steeper slopes have stronger effects
        let gradient_strength = gradient.length();
        
        // Check if player is on the ground
        let sphere_radius = 0.5;
        let was_grounded = physics.grounded;
        physics.grounded = pos.y <= current_height + sphere_radius + 0.01;
        
        // Calculate effective mass (can be adjusted based on gameplay needs)
        let effective_mass = physics.mass * MASS_FACTOR;
        
        // Apply momentum preservation
        if physics.momentum.length_squared() > 0.001 {
            // Gradually blend momentum into velocity
            physics.velocity = physics.velocity.lerp(physics.momentum * (1.0 / effective_mass), 0.2);
        }
        
        // Apply gravity if not grounded
        if !physics.grounded {
            physics.velocity.y -= GRAVITY * delta;
        } else {
            if !was_grounded {
                // Just landed - apply impact and bounce
                let impact = physics.velocity.y.abs();
                if impact > 0.5 {
                    // Bounce based on restitution and impact force
                    physics.velocity.y = impact * RESTITUTION;
                } else {
                    physics.velocity.y = 0.0;
                }
            } else {
                // On ground - roll due to gradient with mass taken into account
                if gradient_strength > 0.001 {
                    // Add force based on terrain gradient (roll downhill)
                    // Steeper slopes cause more acceleration
                    let slope_force = gradient.normalize() * gradient_strength * TERRAIN_SENSITIVITY;
                    
                    // Apply force with consideration for mass
                    let slope_acceleration = slope_force * (GRAVITY / effective_mass);
                    // Apply slope forces gradually to prevent sudden acceleration
                    physics.velocity.x += slope_acceleration.x * delta * 0.7; // Added dampening factor
                    physics.velocity.z += slope_acceleration.z * delta * 0.7; // Added dampening factor
                }
                
                // Apply rolling friction on ground (billiard balls have low friction)
                physics.velocity.x *= FRICTION; 
                physics.velocity.z *= FRICTION;
                
                // Only zero out y velocity when properly grounded
                if physics.velocity.y < 0.0 {
                    physics.velocity.y = 0.0;
                }
            }
        }
        
        // Apply player input force (with mass factored in)
        if physics.grounded && input_direction.length_squared() > 0.0 {
            let input_force = input_direction * (MOVE_SPEED / effective_mass);
            // Reduced multiplier from 5.0 to 2.5
            physics.velocity.x += input_force.x * delta * 2.5;
            physics.velocity.z += input_force.z * delta * 2.5;
        }
        
        // Update momentum
        physics.momentum = physics.momentum.lerp(physics.velocity, 1.0 - MOMENTUM_FACTOR);
        
        // Cap maximum speed for gameplay reasons
        let horiz_speed_squared = physics.velocity.x * physics.velocity.x + physics.velocity.z * physics.velocity.z;
        if horiz_speed_squared > MAX_SPEED * MAX_SPEED {
            let horiz_speed = horiz_speed_squared.sqrt();
            let scale = MAX_SPEED / horiz_speed;
            physics.velocity.x *= scale;
            physics.velocity.z *= scale;
        }
        
        // Apply velocity to position
        transform.translation += physics.velocity * delta;
        
        // Enforce height constraint based on terrain
        let terrain_height = get_terrain_height(transform.translation.x, transform.translation.z);
        let min_height = terrain_height + sphere_radius;
        
        if transform.translation.y < min_height {
            transform.translation.y = min_height;
            physics.grounded = true;
            
            // Adjust velocity when hitting ground
            if physics.velocity.y < 0.0 {
                physics.velocity.y = 0.0;
            }
        }
        
        // Calculate angular velocity based on linear movement
        if physics.grounded && physics.velocity.length() > 0.1 {
            // For a sphere, angular velocity is proportional to linear velocity divided by radius
            // ω = v/r for a rolling sphere
            let move_dir = Vec3::new(physics.velocity.x, 0.0, physics.velocity.z).normalize();
            let right_axis = Vec3::new(-move_dir.z, 0.0, move_dir.x); // Perpendicular to movement
            
            // Angular velocity around the right axis (perpendicular to movement)
            let speed = physics.velocity.length();
            physics.angular_velocity = right_axis * (speed / sphere_radius);
        } else {
            // Gradually reduce angular velocity when not moving
            physics.angular_velocity *= 0.95;
        }
    }
}

// Apply visual rotation to match physics rolling
pub fn apply_physics(
    mut player_query: Query<(&mut Transform, &PlayerPhysics), With<Player>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    for (mut transform, physics) in player_query.iter_mut() {
        // Apply rotation based on angular velocity
        if physics.angular_velocity.length_squared() > 0.001 {
            // Convert angular velocity to a rotation
            let rotation_axis = physics.angular_velocity.normalize();
            let rotation_angle = physics.angular_velocity.length() * delta;
            
            let rotation = Quat::from_axis_angle(rotation_axis, rotation_angle);
            transform.rotation = rotation * transform.rotation;
        }
        
        // Add a slight tilt in the direction of movement on slopes
        if physics.velocity.length() > 0.5 {
            // Calculate tilt angle based on velocity
            let _forward = Vec3::new(physics.velocity.x, 0.0, physics.velocity.z).normalize();
            
            // Only apply subtle tilt (maximum 5 degrees)
            let _tilt_amount = (physics.velocity.length() * 0.03).min(0.09);
            
            // This would tilt the sphere slightly in the direction of movement
            // Commented out because the rotation above already handles rolling
            // We could enable this for additional visual effect if desired
            // let tilt = Quat::from_axis_angle(_forward.cross(Vec3::Y).normalize(), _tilt_amount);
            // transform.rotation = transform.rotation.slerp(tilt, 0.2);
        }
    }
}

// Plugin for the player module
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, move_player)
            // Add physics system running at a fixed timestep for consistent physics
            .add_systems(FixedUpdate, apply_physics);
    }
}
