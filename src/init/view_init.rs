use bevy::prelude::*;

pub struct ViewInitPlugin;

impl Plugin for ViewInitPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, camera_setup);
    }
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
