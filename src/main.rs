mod app;
mod event;
mod ui;
pub mod config;
pub mod domain;
pub mod service;
// mod gh;

use color_eyre::eyre::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui;

use crate::{app::AppState, event::run};

fn main() -> Result<()> {
    let mut state = AppState::new();

    color_eyre::install()?;
    let terminal = ratatui::init();
    enable_raw_mode()?;
    let result = run(terminal, &mut state);
    disable_raw_mode()?;

    ratatui::restore();
    result
}


fn init() {

}