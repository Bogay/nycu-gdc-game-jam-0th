use crate::app::App;

pub mod app;
pub mod color_cycle;
pub mod event;
pub mod fx;
pub mod game;
pub mod setup_logging;
pub mod styling;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    crate::setup_logging::initialize_logging()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
