#[macro_use]
extern crate log;

mod bitboard;
mod game;
mod genius;
mod lichess;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(_) = std::env::var("POIREBOT_LOG") {
        std::env::set_var("POIREBOT_LOG", "info");
    }
    pretty_env_logger::try_init_timed_custom_env("POIREBOT_LOG")
        .expect("Invalid logger configuration");

    lichess::start_bot().await
}
