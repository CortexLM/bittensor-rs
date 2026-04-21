//! bittensor-tui — Terminal UI dashboard for the Bittensor network.
//!
//! Built with ratatui + crossterm, using async tokio channels for
//! non-blocking chain data refresh.

pub mod app;
pub mod event;
pub mod network;
pub mod panels;
pub mod ui;

pub mod prelude {
    //! Re-exports of commonly used types.
    pub use crate::app::{App, Panel};
    pub use crate::event::Event;
    pub use crate::network::NetworkData;
    pub use crate::ui::draw;
}
