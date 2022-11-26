use druid::{AppLauncher, WindowDesc, PlatformError};

mod delegate;
mod controllers;
mod widgets;
mod data;

const DEFAULT_URL: &str = "gemini://gemini.circumlunar.space/";

fn main() -> Result<(), PlatformError> {
    let initial_state = data::CastorState::new(DEFAULT_URL.to_string());
    let window_desc = WindowDesc::new(data::build_ui())
        .title("Castor")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(window_desc)
        .delegate(delegate::Delegate)
        .launch(initial_state)?;
    Ok(())
}