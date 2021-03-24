#[macro_use]
extern crate log;

use anyhow::Context;
use licoricedev::models::board::Event;
use tokio_stream::StreamExt;

mod game;
mod lichess;
mod pieces;
mod genius;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    lichess::start_bot().await
}
