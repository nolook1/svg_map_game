use bevy::prelude::*;

use crate::svg_creation::draw::drawing::DrawPlugin;
use crate::svg_creation::merge_svg::MergeSvgPlugin;
use crate::svg_creation::spatial_grid::SpatialGridPlugin;
use crate::debug_tools::fps_counter::FpsPlugin;
use crate::debug_tools::debug_utils::DebugPlugin;
use crate::init::earth_init::WorldInitPlugin;
use crate::init::view_init::ViewInitPlugin;
use crate::settings::SettingsPlugin;

pub struct GamePlugins;

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SettingsPlugin)
            .add_plugins(FpsPlugin)
            .add_plugins(DrawPlugin)
            .add_plugins(DebugPlugin)
            .add_plugins(MergeSvgPlugin)
            .add_plugins(WorldInitPlugin)
            .add_plugins(ViewInitPlugin)
            .add_plugins(SpatialGridPlugin);
    }
}
