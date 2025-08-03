use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use std::time::Duration;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_text)
            .add_systems(Update, (update, toggle_fps_overlay))
            .init_resource::<FpsCounter>()
            .init_resource::<FpsOverlayState>();
    }
}

#[derive(Resource)]
pub struct FpsCounter {
    pub timer: Timer,
    pub update_now: bool,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            timer: Timer::new(UPDATE_INTERVAL, TimerMode::Repeating),
            update_now: true,
        }
    }
}

#[derive(Resource)]
pub struct FpsOverlayState {
    pub visible: bool,
}

impl Default for FpsOverlayState {
    fn default() -> Self {
        Self { visible: true }
    }
}

pub const FONT_SIZE: f32 = 32.;
pub const FONT_COLOR: Color = Color::BLACK;
pub const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

pub const STRING_FORMAT: &str = "FPS: ";
pub const STRING_INITIAL: &str = "FPS: ...";
pub const STRING_MISSING: &str = "FPS: ???";

#[derive(Component)]
pub struct FpsCounterText;

fn spawn_text(mut commands: Commands) {
    commands
        .spawn((
            Text::new(STRING_INITIAL),
            TextFont {
                font_size: FONT_SIZE,
                ..Default::default()
            },
            TextColor(FONT_COLOR),
        ))
        .insert(FpsCounterText);
}

fn update(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut counter: ResMut<FpsCounter>,
    state: Res<FpsOverlayState>,
    mut query: Query<Entity, With<FpsCounterText>>,
    mut writer: TextUiWriter,
) {
    if !state.visible {
        return;
    }

    if !(counter.update_now || counter.timer.tick(time.delta()).just_finished()) {
        return;
    }

    if counter.timer.paused() {
        for entity in query.iter_mut() {
            writer.text(entity, 0).clear();
        }
    } else {
        let fps = extract_fps(&diagnostics);
        for entity in query.iter_mut() {
            *writer.text(entity, 0) = fps
                .map(|v| format!("{}{:.0}", STRING_FORMAT, v))
                .unwrap_or_else(|| STRING_MISSING.to_string());
        }
    }
}

fn extract_fps(diagnostics: &Res<DiagnosticsStore>) -> Option<f64> {
    diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
}

fn toggle_fps_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<FpsOverlayState>,
    mut commands: Commands,
    query: Query<Entity, With<FpsCounterText>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        state.visible = !state.visible;

        if state.visible {
            // Respawn the text
            spawn_text(commands);
        } else {
            // Despawn the text
            for entity in query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
