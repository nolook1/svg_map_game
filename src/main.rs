use bevy::prelude::*;
use bevy_svg::prelude::*;
use bevy_prototype_lyon::prelude::*;

mod plugins;
mod debug_tools;
mod settings;
mod init;

mod svg_creation;
use svg_creation::*;

use plugins::GamePlugins;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SVGdraw".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((
            SvgPlugin,
            ShapePlugin,
            GamePlugins,
        ))
        .run();
}
