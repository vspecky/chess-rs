extern crate ggez;
mod game;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder, GameResult,
};

use std::path;

const WIN_SIZE: u32 = 800;

fn main() -> GameResult {
    let win_mode = WindowMode::default().dimensions(800., 800.);

    let win_setup = WindowSetup::default().title("Chess.rs");

    let mut asset_path = path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    asset_path.push("src");
    asset_path.push("assets");

    let (mut ctx, mut event_loop) = ContextBuilder::new("Rust Chess", "vSpecky")
        .window_setup(win_setup)
        .window_mode(win_mode)
        .add_resource_path(asset_path)
        .build()
        .unwrap();

    let mut game = game::RChess::new(&mut ctx)?;

    event::run(&mut ctx, &mut event_loop, &mut game)
}
