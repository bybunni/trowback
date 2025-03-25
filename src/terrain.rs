use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use noise::{NoiseFn, Perlin};

// Constants for terrain generation
pub const TERRAIN_SIZE: f32 = 40.0;
pub const TERRAIN_WIDTH: usize = 20;
pub const TERRAIN_HEIGHT: usize = 20;
pub const TERRAIN_HEIGHT_SCALE: f32 = 1.0;
pub const TERRAIN_SEED: u32 = 123;

// Spawn the terrain in the world
pub fn spawn_terrain(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Add procedurally generated terrain
    let terrain_mesh = create_terrain_mesh(
        TERRAIN_WIDTH, 
        TERRAIN_HEIGHT, 
        TERRAIN_SIZE, 
        TERRAIN_HEIGHT_SCALE, 
        TERRAIN_SEED
    );
    
    commands.spawn((
        Mesh3d(meshes.add(terrain_mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(-20.0, -0.5, -20.0),
    ));
}

// Creates a procedurally generated terrain mesh using Perlin noise
pub fn create_terrain_mesh(width: usize, height: usize, size: f32, height_scale: f32, seed: u32) -> Mesh {
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
    
    // Create the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

// Get the height of the terrain at a specific world position
pub fn get_terrain_height(x: f32, z: f32) -> f32 {
    // Make sure coordinates are in local terrain space
    let local_x = x + 20.0;
    let local_z = z + 20.0;
    
    // Check if within bounds
    if local_x >= 0.0 && local_x <= TERRAIN_SIZE && local_z >= 0.0 && local_z <= TERRAIN_SIZE {
        // Calculate terrain height using the same noise function
        let perlin = Perlin::new(TERRAIN_SEED);
        let nx = local_x / TERRAIN_SIZE * 5.0;
        let nz = local_z / TERRAIN_SIZE * 5.0;
        return perlin.get([nx as f64, nz as f64]) as f32 * TERRAIN_HEIGHT_SCALE;
    }
    
    // Default height if outside terrain
    0.0
}

// Plugin for the terrain module
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // No systems to add for terrain yet
        // Could add dynamic terrain features or terrain-related systems later
    }
}
