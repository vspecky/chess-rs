extern crate ggez;
mod game;

use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    graphics, Context, ContextBuilder, GameResult,
};

use std::path;

const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 800;

fn main() -> GameResult {
    let win_mode = WindowMode::default().dimensions(800., 800.);

    let mut asset_path = path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    asset_path.push("src");
    asset_path.push("assets");

    let (mut ctx, mut event_loop) = ContextBuilder::new("Rust Chess", "vSpecky")
        .window_mode(win_mode)
        .add_resource_path(asset_path)
        .build()
        .unwrap();

    let mut game = game::RChess::new(&mut ctx)?;

    event::run(&mut ctx, &mut event_loop, &mut game)
}
