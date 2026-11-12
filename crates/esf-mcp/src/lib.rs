//! MCP server that calculates EVE Online ship fits.

pub mod data;
mod fits;
mod game;
mod server;
mod stats;

pub use game::Game;
pub use server::Server;
