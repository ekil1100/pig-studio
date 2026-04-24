use app_desktop::app::App;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .compact()
        .init();

    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("Pig Studio")
                    .with_inner_size(LogicalSize::new(1440.0, 920.0))
                    .with_min_inner_size(LogicalSize::new(1160.0, 760.0)),
            ),
        )
        .launch(App);
}
