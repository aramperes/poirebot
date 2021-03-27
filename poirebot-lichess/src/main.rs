#[macro_use]
extern crate log;

use std::io::{stdin, Write};
use std::sync::Arc;

use anyhow::Context;
use clap::{App, AppSettings, Arg, ArgMatches};
use licoricedev::client::Lichess;

use crate::bot::{abort_games, send_stockfish_challenge, send_user_challenge, start_bot};

mod bot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .help("Personal authentication token for Lichess")
                .env("LICHESS_TOKEN")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .alias("verbose")
                .help("Turns on verbose logging")
                .required(false)
                .takes_value(false)
                .global(true),
        )
        .subcommand(
            App::new("start")
                .about("Starts the bot to run on Lichess.org")
                .arg(
                    Arg::with_name("challenge")
                        .short("c")
                        .long("challenge")
                        .help("Lichess username to send a challenge to on startup")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("stockfish")
                        .long("stockfish")
                        .help("Start a Stockfish duel with the given strength (1-8)")
                        .takes_value(true)
                        .conflicts_with("challenge")
                        .required(false),
                )
                .arg(
                    Arg::with_name("abort")
                        .long("abort")
                        .help("Aborts all ongoing games")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("no-accept")
                        .long("no-accept")
                        .help("Disables all incoming challenges")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("rematch")
                        .long("rematch")
                        .help("Always challenge for a rematch after game is over")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("upgrade-account")
                .about("Upgrades the Lichess.org account to a BOT account (irreversible)")
                .arg(
                    Arg::with_name("yes")
                        .short("y")
                        .long("yes")
                        .help("Skip the confirmation step")
                        .required(false)
                        .takes_value(false),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    init_logger(args.is_present("debug"));

    let lichess = init_lichess(&args).with_context(|| "Failed to initialize Lichess")?;
    let lichess = Arc::new(lichess);

    let lichess_user = lichess
        .get_my_profile()
        .await
        .with_context(|| "Failed to get current user profile")?;

    if let Some(ref args) = args.subcommand_matches("start") {
        // Abort if specified
        if args.is_present("abort") {
            abort_games(lichess.clone())
                .await
                .with_context(|| "Failed to resign ongoing games")?;
        }

        let stockfish: Option<(u8, u8)> = if let Some(stockfish_arg) = args.value_of("stockfish") {
            let pattern = regex::Regex::new(r"(\d+)(?:-(\d+))?").unwrap();
            let matches = pattern
                .captures(stockfish_arg)
                .with_context(|| "Failed to parse stockfish arg")?;

            let stockfish_min = matches
                .get(1)
                .with_context(|| {
                    "Invalid stockfish arg, use --stockfish=1 or -stockfish=1-8 for example"
                })?
                .as_str();
            let stockfish_min = stockfish_min
                .parse()
                .with_context(|| "Invalid stockfish level")?;
            let stockfish_max = matches
                .get(2)
                .map_or(Ok(stockfish_min), |m| m.as_str().parse())
                .with_context(|| "Invalid max stockfish level")?;
            Some((stockfish_min, stockfish_max))
        } else {
            None
        };

        let config = bot::Config {
            no_accept: args.is_present("no-accept"),
            username: lichess_user.username.clone(),
            rematch: args.is_present("rematch"),
            stockfish: stockfish.map_or(0, |s| s.0),
            stockfish_max: stockfish.map_or(0, |s| s.1),
        };

        // Challenge if specified
        if let Some(challenge_username) = args.value_of("challenge") {
            send_user_challenge(lichess.clone(), challenge_username.into())
                .await
                .with_context(|| format!("Failed to send challenge to {}", challenge_username))?;
        } else if let Some((stockfish, _)) = stockfish {
            send_stockfish_challenge(lichess.clone(), stockfish)
                .await
                .with_context(|| format!("Failed to send challenge to Stockfish"))?;
        }

        start_bot(lichess, config).await
    } else if let Some(ref args) = args.subcommand_matches("upgrade-account") {
        if !args.is_present("yes") {
            println!(
                "Are you sure you want to upgrade {} to a BOT account?",
                &lichess_user.username
            );
            print!("This action is IRREVERSIBLE [y/N]: ");
            std::io::stdout().flush().unwrap();

            let stdin = stdin();
            let mut line = String::new();
            stdin.read_line(&mut line).expect("failed to read stdin");
            if line.trim().to_lowercase() != "y" {
                println!("Aborted");
                return Ok(());
            }
        }
        bot::upgrade_bot_account(lichess, &lichess_user).await
    } else {
        Ok(())
    }
}

fn init_logger(debug: bool) {
    if let Err(_) = std::env::var("POIREBOT_LOG") {
        std::env::set_var("POIREBOT_LOG", "info");
    }
    if debug {
        std::env::set_var("POIREBOT_LOG", "debug");
    }
    pretty_env_logger::try_init_timed_custom_env("POIREBOT_LOG")
        .expect("Invalid logger configuration");
}

fn init_lichess(args: &ArgMatches) -> anyhow::Result<Lichess> {
    let token = args
        .value_of("token")
        .with_context(|| "Missing Lichess token")?
        .to_string();
    Ok(licoricedev::client::Lichess::new(token))
}
