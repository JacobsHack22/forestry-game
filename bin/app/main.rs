//! This example illustrates how to create a button that changes color and text based on its
//! interaction state.

use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use game::data::DataPlugin;
use game::home::HomePlugin;

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    App::new()
        // .insert_resource(ClearColor(Color::rgb(52.0 / 255.0, 59.0 / 255.0, 153.0 / 255.0)))
        .insert_resource(ClearColor(Color::rgb(
            156. / 255.,
            181. / 255.,
            218. / 255.,
        )))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 375.,
                        height: 812.,
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(EasingsPlugin)
        .add_startup_system(setup)
        .add_plugin(DataPlugin)
        .add_plugin(HomePlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
