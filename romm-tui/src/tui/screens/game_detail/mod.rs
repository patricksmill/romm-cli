//! Game detail screen for a single ROM.

mod achievements;
mod cover;
mod render;
mod saves;
mod state;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    DetailTab, GameDetailPrevious, GameDetailScreen, COVER_PANEL_WIDTH_DEFAULT,
    COVER_PANEL_WIDTH_MAX, COVER_PANEL_WIDTH_MIN,
};
