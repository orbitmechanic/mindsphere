use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};

pub fn install_if_requested(app: &mut App) {
    if std::env::var("MINDSPHERE_SCREENSHOT").is_err() {
        return;
    }
    app.add_systems(Update, capture_frame);
}

fn capture_frame(mut commands: Commands, mut frame: Local<u32>, mut exit: MessageWriter<AppExit>) {
    *frame += 1;
    let target = std::env::var("MINDSPHERE_SCREENSHOT_FRAME")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(240);
    if *frame == target {
        let path =
            std::env::var("MINDSPHERE_SCREENSHOT").unwrap_or_else(|_| "mindsphere.png".to_owned());
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path))
            .observe(
                move |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                },
            );
    }
    if *frame > target + 400 {
        exit.write(AppExit::error());
    }
}
