use bevy::prelude::*;
use crate::player::Player;
use crate::camera::MouseLook;
use crate::terrain::get_terrain_height;

// Component for projectiles
#[derive(Component)]
pub struct Projectile {
    // Initial position
    pub start_position: Vec3,
    // Target position
    pub target_position: Vec3,
    // Starting velocity
    pub initial_velocity: Vec3,
    // Lifetime in seconds
    pub lifetime: f32,
    // Current age of projectile
    pub age: f32,
    // Speed multiplier (affects how fast it travels)
    pub speed: f32,
}

// Constants for projectile behavior
const GRAVITY: f32 = 19.6; // Double the normal gravity for heavier feel
const PROJECTILE_LIFETIME: f32 = 8.0; // Extended lifetime since they'll be slower
const PROJECTILE_HEIGHT_FACTOR: f32 = 5.0; // Much higher arc for catapult-like trajectory
const PROJECTILE_SPEED: f32 = 0.25; // Much slower speed for plodding catapult feel

// System to spawn projectiles when mouse is clicked
pub fn spawn_projectile(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    mouse_look: Res<MouseLook>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Only spawn when left mouse button is just pressed and we have a valid target
    if mouse_input.just_pressed(MouseButton::Left) && mouse_look.is_initialized {
        // Get player position (if available)
        if let Ok(player_transform) = player_query.get_single() {
            let player_pos = player_transform.translation;
            let target_pos = mouse_look.target_position;
            
            // Calculate horizontal distance to target
            let horizontal_dist = Vec3::new(
                target_pos.x - player_pos.x, 
                0.0, 
                target_pos.z - player_pos.z
            ).length();
            
            // Calculate height difference
            let height_diff = target_pos.y - player_pos.y;
            
            // Calculate projectile trajectory (high arching ballistic path)
            // Starting position is slightly above the player
            let start_pos = player_pos + Vec3::new(0.0, 0.3, 0.0);
            
            // Calculate velocity for ballistic trajectory
            // We'll use physics formulas for projectile motion to create a nice arc
            
            // Direction to target (horizontal only)
            let direction = Vec3::new(
                target_pos.x - player_pos.x,
                0.0,
                target_pos.z - player_pos.z
            ).normalize();
            
            // Desired max height above higher of start/end points
            let max_height_above = horizontal_dist * PROJECTILE_HEIGHT_FACTOR;
            
            // Calculate initial velocity for a much slower, higher arc
            // Using simplified ballistic equations for a catapult-like trajectory
            // Make travel time much longer for slower projectiles
            let travel_time = (horizontal_dist / PROJECTILE_SPEED).max(3.0);
            
            // Horizontal component of velocity
            let horizontal_velocity = direction * (horizontal_dist / travel_time);
            
            // Vertical component for arc (solving quadratic equation for projectile motion)
            // v_y = (h + 0.5*g*t²)/t
            let vertical_velocity = (
                height_diff + 
                max_height_above + 
                0.5 * GRAVITY * travel_time * travel_time
            ) / travel_time;
            
            // Combine into 3D velocity
            let initial_velocity = Vec3::new(
                horizontal_velocity.x,
                vertical_velocity,
                horizontal_velocity.z
            );
            
            // Create larger, boulder-like projectile for catapult feel
            let arrow_mesh = Mesh::from(Sphere::new(0.15));
            
            // Create stone-like material for catapult boulder appearance
            let arrow_material = StandardMaterial {
                base_color: Color::srgb(0.4, 0.4, 0.4),
                emissive: Color::srgb(0.0, 0.0, 0.0).into(),
                perceptual_roughness: 0.9,
                metallic: 0.0,
                reflectance: 0.05,
                ..default()
            };
            
            // Apply a random slight variation to initial velocity for natural feel
            let variation = 0.05;
            let random_variation = Vec3::new(
                (rand::random::<f32>() - 0.5) * variation,
                (rand::random::<f32>()) * variation, // Slight positive bias on Y
                (rand::random::<f32>() - 0.5) * variation
            );
            let initial_velocity = initial_velocity + random_variation;
            
            // Spawn projectile entity
            commands.spawn((
                Projectile {
                    start_position: start_pos,
                    target_position: target_pos,
                    initial_velocity,
                    lifetime: PROJECTILE_LIFETIME,
                    age: 0.0,
                    speed: PROJECTILE_SPEED,
                },
                Mesh3d(meshes.add(arrow_mesh)),
                MeshMaterial3d(materials.add(arrow_material)),
                Transform::from_translation(start_pos),
                Name::new("Catapult Boulder"),
            ));
        }
    }
}

// System to update projectile positions with physics
pub fn update_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut projectile) in projectile_query.iter_mut() {
        // Update projectile age
        projectile.age += time.delta_secs();
        
        // Remove if lifetime exceeded
        if projectile.age >= projectile.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Calculate current position based on ballistic trajectory
        let t = projectile.age;
        let initial_vel = projectile.initial_velocity;
        let start_pos = projectile.start_position;
        
        // Apply ballistic motion formula: pos = start_pos + initial_vel*t + 0.5*gravity*t²
        let current_pos = Vec3::new(
            start_pos.x + initial_vel.x * t,
            start_pos.y + initial_vel.y * t - 0.5 * GRAVITY * t * t,
            start_pos.z + initial_vel.z * t
        );
        
        // Update transform position
        transform.translation = current_pos;
        
        // Orient projectile to face in the direction of travel
        if t > 0.0 {
            // Calculate current velocity vector (derivative of position)
            let current_velocity = Vec3::new(
                initial_vel.x,
                initial_vel.y - GRAVITY * t,
                initial_vel.z
            );
            
            // Only update rotation if moving
            if current_velocity.length_squared() > 0.001 {
                // Make the projectile point in the direction it's moving
                transform.look_to(current_velocity.normalize(), Vec3::Y);
                
                // Add a slight roll based on arc direction
                let roll_angle = (t * 2.0).sin() * 0.2; // Small oscillating roll
                let roll = Quat::from_rotation_z(roll_angle);
                transform.rotation = transform.rotation * roll;
            }
        }
        
        // Check for collision with terrain
        let terrain_height = get_terrain_height(transform.translation.x, transform.translation.z);
        if transform.translation.y <= terrain_height {
            // Position the arrow at the terrain with slight embedding
            transform.translation.y = terrain_height;
            
            // Adjust rotation to stick into the ground
            let up_vector = Vec3::Y;
            let normal_vector = Vec3::new(0.0, 1.0, 0.0); // Simplified - assume flat terrain
            
            // Face slightly into the ground
            let impact_direction = transform.rotation * Vec3::Z;
            let ground_direction = impact_direction.lerp(normal_vector, 0.5).normalize();
            transform.look_to(ground_direction, up_vector);
            
            // Let arrows stay for a while after impact
            projectile.lifetime = projectile.age + 10.0; // Stay for 10 more seconds
            
            // Make it a "static" projectile by flagging it
            projectile.speed = 0.0;
        }
    }
}

// Plugin for projectile functionality
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, spawn_projectile)
            .add_systems(Update, update_projectiles.after(spawn_projectile));
    }
}
