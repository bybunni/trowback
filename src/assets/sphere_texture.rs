use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// Generate a simple procedural texture for the sphere
// This creates a simple billiard ball style texture with colored segments
pub fn create_sphere_texture() -> Image {
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
    Image::new_fill(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &rgba,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
}
