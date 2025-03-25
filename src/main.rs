use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use noise::{NoiseFn, Perlin};

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

    // Add procedurally generated terrain
    let terrain_mesh = create_terrain_mesh(20, 20, 40.0, 1.0, 123);
    commands.spawn((
        Mesh3d(meshes.add(terrain_mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(-20.0, -0.5, -20.0),
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

// Creates a procedurally generated terrain mesh using Perlin noise
fn create_terrain_mesh(width: usize, height: usize, size: f32, height_scale: f32, seed: u32) -> Mesh {
    // Number of vertices
    let vertex_count = (width + 1) * (height + 1);
    
    // Create a Perlin noise generator with the specified seed
    let perlin = Perlin::new(seed);
    
    // Create the terrain vertices
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    
    // Generate the vertices grid
    for z in 0..=height {
        for x in 0..=width {
            // Calculate normalized coordinates for noise
            let nx = x as f64 / width as f64 * 5.0;  // Scale factor for noise frequency
            let nz = z as f64 / height as f64 * 5.0;
            
            // Get terrain height from noise
            let y = perlin.get([nx, nz]) as f32 * height_scale;
            
            // Calculate world-space coordinates
            let pos_x = x as f32 / width as f32 * size;
            let pos_z = z as f32 / height as f32 * size;
            
            // Add the vertex position
            positions.push([pos_x, y, pos_z]);
            
            // Calculate approximate normals (will be smoothed later)
            // For simplicity, we use an up vector initially
            normals.push([0.0, 1.0, 0.0]);
            
            // Add texture coordinates
            uvs.push([x as f32 / width as f32, z as f32 / height as f32]);
        }
    }
    
    // Create the triangle indices
    let mut indices = Vec::with_capacity(width * height * 6); // 2 triangles per grid cell, 3 vertices per triangle
    
    for z in 0..height {
        for x in 0..width {
            // Calculate the indices of the four corners of the current grid cell
            let tl = z * (width + 1) + x;
            let tr = tl + 1;
            let bl = (z + 1) * (width + 1) + x;
            let br = bl + 1;
            
            // Add the two triangles for this grid cell
            indices.push(tl as u32);
            indices.push(bl as u32);
            indices.push(tr as u32);
            
            indices.push(tr as u32);
            indices.push(bl as u32);
            indices.push(br as u32);
        }
    }
    
    // Calculate better normals by averaging the normals of adjacent triangles
    // This is a simple approach - each vertex normal is the average of the normals
    // of all triangles it belongs to
    let mut normal_sums = vec![[0.0, 0.0, 0.0]; vertex_count];
    let mut normal_counts = vec![0; vertex_count];
    
    // For each triangle, calculate its normal and add it to each vertex
    for i in (0..indices.len()).step_by(3) {
        let idx0 = indices[i] as usize;
        let idx1 = indices[i + 1] as usize;
        let idx2 = indices[i + 2] as usize;
        
        let v0 = Vec3::from(positions[idx0]);
        let v1 = Vec3::from(positions[idx1]);
        let v2 = Vec3::from(positions[idx2]);
        
        // Calculate the triangle normal using cross product
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();
        
        // Add the normal to each vertex of the triangle
        for &idx in &[idx0, idx1, idx2] {
            normal_sums[idx][0] += normal.x;
            normal_sums[idx][1] += normal.y;
            normal_sums[idx][2] += normal.z;
            normal_counts[idx] += 1;
        }
    }
    
    // Normalize all the normals
    for i in 0..vertex_count {
        if normal_counts[i] > 0 {
            let count = normal_counts[i] as f32;
            let mut normal = Vec3::new(
                normal_sums[i][0] / count,
                normal_sums[i][1] / count,
                normal_sums[i][2] / count,
            );
            normal = normal.normalize();
            normals[i] = [normal.x, normal.y, normal.z];
        }
    }
    
    // Create the mesh with the correct Bevy API
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}