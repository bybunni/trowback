use bevy::prelude::*;
use crate::player::Player;

// Component for tracking the camera that follows the player
#[derive(Component)]
pub struct FollowCamera;

// Setup the camera
pub fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        Camera3d::default(),
        FollowCamera,
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// Camera follows the player with a slight offset
pub fn camera_follow(
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

// Plugin for the camera module
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow);
    }
}
