use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::{App, Arg};
use licoricedev::client::Lichess;
use licoricedev::models::board::Challengee::LightUser;
use licoricedev::models::board::{Challenge, Event, GameID};
use licoricedev::models::game::Game;
use tokio::sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;

/// The world containing all games.
#[derive(Default)]
struct World {
    /// Tasks: (game ID, sender)
    games: HashMap<String, UnboundedSender<Message>>,
}

/// Messages to pass to and from tasks.
#[derive(Debug, Clone)]
enum Message {
    /// Instruct to process the challenge (can accept or reject).
    NewChallenge(Challenge),
    /// Instruct to process the new game.
    NewGame(GameID),
    /// Game/challenge is aborted.
    Abort(GameID),
}

/// Task that handles new game state messages.
async fn message_loop(recv: &mut UnboundedReceiver<Message>, lichess: Arc<Lichess>) {
    while let Some(message) = recv.recv().await {
        match message {
            Message::NewChallenge(challenge) => {
                let challenger_name = challenge.challenger.clone().unwrap().username;
                info!(
                    "Challenge received: {} (other: {})",
                    challenge.id, challenger_name,
                );

                let accepted = accept_or_decline_challenge(&challenge, lichess.clone()).await;
                if accepted {
                    info!(
                        "Challenge accepted: {} (other: {})",
                        challenge.id, challenger_name,
                    );
                } else {
                    info!(
                        "Challenge declined: {} (other: {})",
                        challenge.id, challenger_name,
                    );
                    break;
                }
            }
            Message::NewGame(game_id) => {
                info!("Game starting: {}", game_id.id);
            }
            Message::Abort(game_id) => {
                info!("Game/Challenge aborted: {}", game_id.id);
                break;
            }
        }
    }
}

/// Decides to accept or deline the challenge and sends the response.
/// Returns the new game if accepted, false otherwise.
async fn accept_or_decline_challenge(challenge: &Challenge, lichess: Arc<Lichess>) -> bool {
    // TODO: Determine acceptable modes/time.
    // Decline
    // if let Err(e) = lichess
    //     .challenge_decline(
    //         &challenge.id,
    //         Some("Sorry, I cannot play under these conditions."),
    //     )
    //     .await
    //     .with_context(|| "Failed to Decline challenge")
    // {
    //     error!("{:?}", e);
    // }
    // false

    // Accept
    if let Err(e) = lichess
        .challenge_accept(&challenge.id)
        .await
        .with_context(|| "Failed to accept challenge")
    {
        error!("{:?}", e);
    }
    true
}

/// Handles a new challenge by creating a new task with communication channel.
async fn handle_new_challenge(
    challenge: Challenge,
    world: &mut World,
    lichess: Arc<Lichess>,
) -> anyhow::Result<()> {
    let game_id = challenge.id.clone();
    let challenger = challenge.challenger.clone().unwrap().username;

    if challenger == "poirebot" {
        // Ignore challenges from self
        return Ok(());
    }

    let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel::<Message>();
    world.games.insert(game_id.clone(), sender);

    tokio::spawn(async move { message_loop(&mut recv, lichess.clone()).await });

    let sender = world.games.get(&game_id).unwrap();
    sender
        .send(Message::NewChallenge(challenge))
        .unwrap_or_else(|e| error!("Failed to dispatch NewChallenge: {:?}", e));

    Ok(())
}

/// Handles a new game by creating a new task with communication channel.
async fn handle_new_game(
    game_id: GameID,
    world: &mut World,
    lichess: Arc<Lichess>,
) -> anyhow::Result<()> {
    let id = game_id.clone().id;

    // If there is already a challenge task, abort it
    if world.games.contains_key(&id) {
        abort_task(&id, world).await;
    }

    let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel::<Message>();
    world.games.insert(id.clone(), sender);

    tokio::spawn(async move { message_loop(&mut recv, lichess.clone()).await });

    let sender = world.games.get(&id).unwrap();
    sender
        .send(Message::NewGame(game_id))
        .unwrap_or_else(|e| error!("Failed to dispatch NewGame: {:?}", e));

    Ok(())
}

/// Dispatches the 'Abort' message to the game, closing it.
async fn abort_task(game_id: &str, world: &mut World) {
    if let Some(sender) = world.games.get(game_id) {
        sender
            .send(Message::Abort(GameID { id: game_id.into() }))
            .unwrap_or_else(|_| ());
        world.games.remove(game_id);
    } else {
        warn!(
            "Tried to abort non-existing task: {}. Nothing happens.",
            game_id
        );
    }
}

async fn process_incoming_event(
    event: Event,
    world: &mut World,
    lichess: Arc<Lichess>,
) -> anyhow::Result<()> {
    debug!("Received incoming event: {:?}", event);

    match event {
        Event::Challenge { challenge } => handle_new_challenge(challenge, world, lichess)
            .await
            .with_context(|| "Failed to dispatch new challenge"),

        Event::ChallengeCanceled { challenge } => Ok(abort_task(&challenge.id, world).await),

        Event::ChallengeDeclined { challenge } => Ok(abort_task(&challenge.id, world).await),

        Event::GameFinish { game } => Ok(abort_task(&game.id, world).await),

        Event::GameStart { game: game_id } => handle_new_game(game_id, world, lichess).await,
    }
}

async fn send_challenge(lichess: Arc<Lichess>, username: String) -> anyhow::Result<()> {
    // TODO: Pass some of these from the CLI
    let options = [
        ("rated", "false"),
        ("clock.limit", "300"),
        ("clock.increment", "0"),
        ("color", "random"),
        ("variant", "standard"),
    ];
    lichess
        .challenge_create(&username, Some(&options))
        .await
        .with_context(|| "Failed to create challenge")
        .map(|challenge| {
            info!("Sent challenge to {}", &username);
        })
}

pub async fn start_bot() -> anyhow::Result<()> {
    // let token = std::env::var("LICHESS_TOKEN").with_context(|| "Missing LICHESS_TOKEN")?;
    // debug!("Using Lichess token: {}", token);

    let args = App::new("poirebot")
        .version("0.1.0")
        .author("Aram Peres <contact@aramperes.ca>")
        .about("A chess bot")
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
            Arg::with_name("challenge")
                .short("c")
                .long("challenge")
                .help("Lichess username to send a challenge to on startup")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    let token = args
        .value_of("token")
        .with_context(|| "Missing Lichess token")?
        .to_string();

    let lichess = licoricedev::client::Lichess::new(token);
    let mut event_stream = lichess
        .stream_incoming_events()
        .await
        .with_context(|| "Failed to get incoming events")?;

    let lichess = Arc::new(lichess);
    let mut world = World::default();

    // Challenge if specified
    if let Some(challenge_username) = args.value_of("challenge") {
        send_challenge(lichess.clone(), challenge_username.into())
            .await
            .with_context(|| format!("Failed to send challenge to {}", challenge_username))?;
    }

    while let Some(event) = event_stream.next().await {
        if let Ok(event) = event {
            if let Err(e) = process_incoming_event(event, &mut world, lichess.clone()).await {
                error!("Failed to process event: {:?}", e);
            }
        } else {
            info!("Stopped processing events from loop.");
            break;
        }
    }

    Ok(())
}
