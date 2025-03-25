use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::utils::HashMap;
use noise::{NoiseFn, Perlin};

// Constants for terrain generation
pub const CHUNK_SIZE: f32 = 40.0;
pub const CHUNK_RESOLUTION: usize = 24; // Higher resolution for more detailed terrain
pub const TERRAIN_HEIGHT_SCALE: f32 = 8.0; // Increased height for more dramatic hills
pub const TERRAIN_SEED: u32 = 123;

// Additional noise parameters for varied terrain
pub const MAIN_NOISE_SCALE: f64 = 80.0; // Base scale for primary features
pub const DETAIL_NOISE_SCALE: f64 = 30.0; // Scale for secondary details
pub const TERTIARY_NOISE_SCALE: f64 = 10.0; // Scale for small details

// Component to mark terrain chunks
#[derive(Component)]
pub struct TerrainChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

// Resource to track loaded chunks
#[derive(Resource)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32), Entity>,
    pub material_handle: Handle<StandardMaterial>,
}

// System to spawn initial terrain
pub fn spawn_initial_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a default green material for all terrain chunks
    let material_handle = materials.add(Color::srgb(0.3, 0.5, 0.3));
    
    // Create the chunk manager resource
    commands.insert_resource(ChunkManager {
        loaded_chunks: HashMap::new(),
        material_handle: material_handle.clone(),
    });
    
    // Spawn the initial 3x3 grid of chunks
    for z in -1..=1 {
        for x in -1..=1 {
            spawn_terrain_chunk(&mut commands, &mut meshes, material_handle.clone(), x, z);
        }
    }
}

// Creates a procedurally generated terrain mesh for a specific chunk
pub fn create_terrain_mesh(chunk_x: i32, chunk_z: i32) -> Mesh {
    // Constants for mesh generation
    let width = CHUNK_RESOLUTION;
    let height = CHUNK_RESOLUTION;
    let size = CHUNK_SIZE;
    
    // Number of vertices
    let vertex_count = (width + 1) * (height + 1);
    
    // We don't need to create a perlin noise instance here since we use get_terrain_height
    
    // Create the terrain vertices
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    
    // Generate the vertices grid
    for z in 0..=height {
        for x in 0..=width {
            // Calculate world position for this vertex
            let world_x = chunk_x as f32 * size + x as f32 / width as f32 * size;
            let world_z = chunk_z as f32 * size + z as f32 / height as f32 * size;
            
            // Use the global height function to ensure consistency across chunks
            let y = get_terrain_height(world_x, world_z);
            
            // Add the vertex position relative to chunk origin
            positions.push([x as f32 / width as f32 * size, y, z as f32 / height as f32 * size]);
            
            // Calculate approximate normals (will be smoothed later)
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
    
    // Create the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

// Get the height of the terrain at any world position
pub fn get_terrain_height(x: f32, z: f32) -> f32 {
    // Create Perlin noise generators with different seeds for variety
    let perlin_main = Perlin::new(TERRAIN_SEED);
    let perlin_detail = Perlin::new(TERRAIN_SEED + 42);
    let perlin_tertiary = Perlin::new(TERRAIN_SEED + 123);
    
    // Calculate coordinates at different scales
    let nx_main = x as f64 / MAIN_NOISE_SCALE;
    let nz_main = z as f64 / MAIN_NOISE_SCALE;
    
    let nx_detail = x as f64 / DETAIL_NOISE_SCALE;
    let nz_detail = z as f64 / DETAIL_NOISE_SCALE;
    
    let nx_tertiary = x as f64 / TERTIARY_NOISE_SCALE;
    let nz_tertiary = z as f64 / TERTIARY_NOISE_SCALE;
    
    // Main terrain features (rolling hills) - larger scale
    let main_height = perlin_main.get([nx_main, nz_main]) as f32;
    
    // Secondary details - medium scale features
    let detail_height = perlin_detail.get([nx_detail, nz_detail]) as f32 * 0.3;
    
    // Small terrain details - small bumps and texture
    let tertiary_height = perlin_tertiary.get([nx_tertiary, nz_tertiary]) as f32 * 0.1;
    
    // Combine all features with varied weights
    let combined_height = main_height + detail_height + tertiary_height;
    
    // Apply a slight exponential curve to create more dramatic hills and flatter valleys
    let height_curve = (combined_height + 1.0) * 0.5; // Normalize to 0-1 range
    let curved_height = height_curve.powf(1.3) * 2.0 - 1.0; // Apply curve and rescale
    
    return curved_height * TERRAIN_HEIGHT_SCALE;
}

// Function to spawn a single terrain chunk at the given coordinates
pub fn spawn_terrain_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    chunk_x: i32,
    chunk_z: i32,
) -> Entity {
    // Calculate world position for this chunk
    let position_x = chunk_x as f32 * CHUNK_SIZE;
    let position_z = chunk_z as f32 * CHUNK_SIZE;
    
    // Create mesh for this specific chunk
    let chunk_mesh = create_terrain_mesh(chunk_x, chunk_z);
    
    // Spawn the chunk entity
    let chunk_entity = commands.spawn((
        TerrainChunk { chunk_x, chunk_z },
        Mesh3d(meshes.add(chunk_mesh)),
        MeshMaterial3d(material),
        Transform::from_xyz(position_x, 0.0, position_z),
    )).id();
    
    chunk_entity
}

// System to manage terrain chunks based on player position
pub fn manage_terrain_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    // Get player position
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;
        
        // Calculate which chunk the player is in
        let current_chunk_x = (player_pos.x / CHUNK_SIZE).floor() as i32;
        let current_chunk_z = (player_pos.z / CHUNK_SIZE).floor() as i32;
        
        // Define the radius of chunks to keep loaded (in chunk coordinates)
        let chunk_radius = 2; // Keep 5x5 grid of chunks around player (2 in each direction + current)
        
        // Determine which chunks should be loaded
        let mut chunks_to_load = Vec::new();
        for z in (current_chunk_z - chunk_radius)..=(current_chunk_z + chunk_radius) {
            for x in (current_chunk_x - chunk_radius)..=(current_chunk_x + chunk_radius) {
                let chunk_key = (x, z);
                if !chunk_manager.loaded_chunks.contains_key(&chunk_key) {
                    chunks_to_load.push(chunk_key);
                }
            }
        }
        
        // Spawn new chunks as needed
        for (x, z) in chunks_to_load {
            let new_chunk = spawn_terrain_chunk(
                &mut commands,
                &mut meshes,
                chunk_manager.material_handle.clone(),
                x,
                z
            );
            chunk_manager.loaded_chunks.insert((x, z), new_chunk);
        }
        
        // Optional: unload chunks that are too far away
        // This can be implemented later if necessary
    }
}

// Plugin for the terrain module
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ChunkManager {
                loaded_chunks: HashMap::new(),
                material_handle: Handle::default(),
            })
            .add_systems(Startup, spawn_initial_terrain)
            .add_systems(Update, manage_terrain_chunks);
    }
}
