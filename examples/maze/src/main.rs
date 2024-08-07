use bevy::prelude::*;
use bevy_bsml::prelude::*;
use maze::{
    bit_gossip::{BitGossipPlugin, MyGraph},
    MazePlugin,
};

const WINDOW_SCALE: f32 = 3.5;
const WINDOW_SIZE: Vec2 = Vec2::new(480. * WINDOW_SCALE, 270. * WINDOW_SCALE);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "maze".to_string(),
                        resolution: WINDOW_SIZE.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(BsmlPlugin)
        .add_plugins((MazePlugin::new(80, 80), BitGossipPlugin))
        .add_systems(Startup, spawn_hud)
        .add_systems(Update, update_hud_text)
        .run();
}

#[derive(Component)]
pub struct HudText(pub f32);

fn update_hud_text(
    time: Res<Time>,
    graph: Query<Ref<MyGraph>>,
    mut hud_text: Query<(&mut HudText, &mut Text), With<HudText>>,
) {
    let res = graph.get_single();

    if let Ok(graph) = res {
        if graph.is_added() {
            for (mut timer, mut text) in hud_text.iter_mut() {
                timer.0 += time.delta_seconds();
                text.as_mut().sections[0].value = format!("Graph Built {:.2}s", timer.0);
            }
        }
        return;
    }

    for (mut timer, mut text) in hud_text.iter_mut() {
        timer.0 += time.delta_seconds();
        text.as_mut().sections[0].value = format!("Building Graph {:.2}s", timer.0);
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn_bsml(
        bsml! {(node class=[W_FULL, H_FULL, JUSTIFY_CENTER, ITEMS_START, BG_TRANSPARENT]) {
            (text labels=[HudText(0.)] class=[TEXT_LG, TEXT_WHITE]) { "Building Graph 0.00s" }
        }},
    );
}
