#[macro_use]
extern crate log;

mod bitboard;
mod game;
mod genius;
mod lichess;
mod pieces;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    lichess::start_bot().await
}
