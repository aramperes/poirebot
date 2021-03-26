use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use licoricedev::client::Lichess;
use licoricedev::models::board::{BoardState, Challenge, Challengee, Event, GameFull, GameID};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio_stream::StreamExt;

use poirebot::game::pieces::Color;
use poirebot::game::{Move, TurnCounter};
use poirebot::genius::Brain;

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
    NewGame,
    /// Game/challenge is aborted.
    Abort,
    /// Game/challenge is aborted.
    BoardChat(String, String),
    /// When the opponent has completed a move. The string is the description of the move.
    OpponentMove(String),
    /// When it's our turn to move.
    BotMove,
    /// Updates the color of the bot.
    BotColor(Color),
}

/// Configures the bot.
#[derive(Debug, Clone)]
pub struct Config {
    /// The Lichess.org username of the bot.
    pub username: String,
    /// If true, does not accept any incoming challenge.
    pub no_accept: bool,
}

/// Task that handles new game state messages.
async fn message_loop(
    game_id: GameID,
    recv: &mut UnboundedReceiver<Message>,
    lichess: Arc<Lichess>,
    config: &Config,
) {
    let mut brain = Brain::default();

    while let Some(message) = recv.recv().await {
        match message {
            Message::NewChallenge(challenge) => {
                let challenger_name = challenge.challenger.clone().unwrap().username;
                info!(
                    "Challenge received: {} (other: {})",
                    challenge.id, challenger_name,
                );

                let accepted = accept_or_decline_challenge(&challenge, lichess.clone(), &config)
                    .await
                    .with_context(|| "Failed to accept/decline challenge")
                    .unwrap_or_else(|e| {
                        error!("{:?}", e);
                        false
                    });
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
            Message::NewGame => {
                let id = game_id.id.clone();
                info!("Game starting: {}", &id);
                lichess
                    .write_in_bot_chat(&id, "player", "Welcome to the poire zone")
                    .await
                    .unwrap_or_else(|_| ());
            }
            Message::Abort => {
                info!("Game/Challenge aborted: {}", game_id.id);
                break;
            }
            Message::BoardChat(username, message) => {
                info!("({})\t\t{}\t\t{}", game_id.id, username, message);
            }
            Message::OpponentMove(m) => {
                info!("Opponent ({}) moved: {}", game_id.id, m);
                brain.opponent_move(Move::from_pure_notation(&m));
            }
            Message::BotMove => {
                info!("Our turn! ({})", game_id.id);
                let (sensor, recv) = oneshot::channel::<Option<Move>>();
                brain.choose_move(sensor);

                let m = recv.await.unwrap();
                if let Some(m) = m {
                    if let Err(e) = lichess
                        .make_a_bot_move(&game_id.id, m.to_pure_notation().as_str(), false)
                        .await
                        .with_context(|| "Failed to dispatch move to Lichess")
                    {
                        error!("{:?}", e);
                        lichess
                            .resign_bot_game(&game_id.id)
                            .await
                            .unwrap_or_else(|_| ());
                        break;
                    }
                    brain.own_move(m);
                } else {
                    error!("ran out of moves ({})", game_id.id);
                    lichess
                        .resign_bot_game(&game_id.id)
                        .await
                        .unwrap_or_else(|_| ());
                }
            }
            Message::BotColor(color) => {
                brain.color = color;
            }
        }
    }
}

/// Decides to accept or decline the challenge and sends the response.
/// Returns the new game if accepted, false otherwise.
async fn accept_or_decline_challenge(
    challenge: &Challenge,
    lichess: Arc<Lichess>,
    config: &Config,
) -> anyhow::Result<bool> {
    // TODO: Determine acceptable modes/time.
    let accept = !config.no_accept;

    if accept {
        lichess
            .challenge_accept(&challenge.id)
            .await
            .map(|_| true)
            .with_context(|| "Failed to accept challenge")
    } else {
        lichess
            .challenge_decline(
                &challenge.id,
                Some("Sorry, I cannot play under these conditions."),
            )
            .await
            .map(|_| false)
            .with_context(|| "Failed to decline challenge")
    }
}

/// Handles a new challenge by creating a new task with communication channel.
async fn handle_new_challenge(
    challenge: Challenge,
    world: &mut World,
    lichess: Arc<Lichess>,
    config: &Config,
) -> anyhow::Result<()> {
    let game_id = challenge.id.clone();
    let challenger = challenge.challenger.clone().unwrap().username;

    if challenger == config.username {
        // Ignore challenges from self
        return Ok(());
    }

    let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel::<Message>();
    world.games.insert(game_id.clone(), sender.clone());

    let game_id = GameID {
        id: game_id.clone(),
    };
    let config = config.clone();
    tokio::spawn(async move { message_loop(game_id, &mut recv, lichess.clone(), &config).await });

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
    config: &Config,
) -> anyhow::Result<()> {
    let id = game_id.clone().id;
    let (config_a, config_b) = (config.clone(), config.clone());
    let (lichess_a, lichess_b) = (lichess.clone(), lichess.clone());

    // If there is already a challenge task, abort it
    if world.games.contains_key(&id) {
        abort_task(&id, world).await;
    }

    let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel::<Message>();
    // Replaces any existing communication
    world.games.insert(id.clone(), sender.clone());

    tokio::spawn(
        async move { message_loop(game_id, &mut recv, lichess_a.clone(), &config_a).await },
    );

    sender
        .send(Message::NewGame)
        .unwrap_or_else(|e| error!("Failed to dispatch NewGame: {:?}", e));

    // Start consuming board events
    tokio::spawn(async move {
        let event_stream = lichess_b
            .clone()
            .stream_bot_game_state(&id)
            .await
            .with_context(|| "Failed to get board event stream");

        let mut turn_counter = TurnCounter::default();

        match event_stream {
            Ok(mut event_stream) => {
                while let Some(event) = event_stream.next().await {
                    if let Ok(event) = event {
                        dispatch_board_event(&sender, &id, &mut turn_counter, event, &config_b)
                            .await;
                    }
                }
                info!("Stopped receiving events from board loop: {}", &id);
            }
            Err(e) => error!("{:?}", e),
        }
    });

    Ok(())
}

async fn dispatch_board_event(
    sender: &UnboundedSender<Message>,
    id: &str,
    turn_counter: &mut TurnCounter,
    board_state: BoardState,
    config: &Config,
) {
    debug!("Board event ({}): {:#?}", id, board_state);
    match board_state {
        BoardState::ChatLine(chat_line) => {
            let username = chat_line.username;
            if username != config.username {
                sender
                    .send(Message::BoardChat(username, chat_line.text))
                    .unwrap_or_else(|_| ());
            }
        }
        BoardState::GameFull(state) => {
            if state.state.status == "started" {
                if turn_counter.first_move {
                    let white = is_bot_white(&state, &config.username);
                    turn_counter.our_turn = white;
                    let color = if white { Color::White } else { Color::Black };
                    sender.send(Message::BotColor(color)).unwrap_or_else(|_| ());
                }
                if turn_counter.our_turn {
                    sender.send(Message::BotMove).unwrap_or_else(|_| ());
                } else if state.state.moves.is_empty() {
                    // We're still waiting for the opponent's move.
                } else {
                    let m = last_move(&state.state.moves);
                    sender.send(Message::OpponentMove(m)).unwrap_or_else(|_| ());
                    sender.send(Message::BotMove).unwrap_or_else(|_| ());
                }
                turn_counter.next();
            } else {
                warn!("Unhandled board status: {}", state.state.status);
            }
        }
        BoardState::GameState(state) => {
            if state.status == "started" {
                if turn_counter.first_move {
                    error!("Did not expect first move to be partial game state.");
                } else if turn_counter.our_turn {
                    let m = last_move(&state.moves);
                    sender.send(Message::OpponentMove(m)).unwrap_or_else(|_| ());
                    sender.send(Message::BotMove).unwrap_or_else(|_| ());
                }
                turn_counter.next();
            } else {
                warn!("Unhandled board status: {}", state.status);
            }
        }
    }
}

/// Dispatches the 'Abort' message to the game, closing it.
async fn abort_task(game_id: &str, world: &mut World) {
    if let Some(sender) = world.games.get(game_id) {
        sender.send(Message::Abort).unwrap_or_else(|_| ());
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
    config: &Config,
) -> anyhow::Result<()> {
    debug!("Received incoming event: {:?}", event);

    match event {
        Event::Challenge { challenge } => handle_new_challenge(challenge, world, lichess, config)
            .await
            .with_context(|| "Failed to dispatch new challenge"),

        Event::ChallengeCanceled { challenge } => Ok(abort_task(&challenge.id, world).await),

        Event::ChallengeDeclined { challenge } => Ok(abort_task(&challenge.id, world).await),

        Event::GameFinish { game } => Ok(abort_task(&game.id, world).await),

        Event::GameStart { game: game_id } => {
            handle_new_game(game_id, world, lichess, config).await
        }
    }
}

fn is_bot_white(game_full: &GameFull, bot_username: &str) -> bool {
    let white = &game_full.white;
    match white {
        Challengee::LightUser(user) => user.username == bot_username,
        Challengee::StockFish(_) => false,
    }
}

fn last_move(moves: &str) -> String {
    moves.split(' ').last().unwrap_or_default().to_string()
}

pub async fn send_user_challenge(lichess: Arc<Lichess>, username: String) -> anyhow::Result<()> {
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
            info!(
                "Sent challenge to {}: https://lichess.org/{}",
                &username,
                challenge.challenge.unwrap().id
            );
        })
}

pub async fn send_stockfish_challenge(lichess: Arc<Lichess>, level: u8) -> anyhow::Result<()> {
    // TODO: Pass some of these from the CLI
    let options = [
        ("rated", "false"),
        ("clock.limit", "300"),
        ("clock.increment", "0"),
        ("color", "random"),
        ("variant", "standard"),
    ];
    lichess
        .challenge_stockfish(level, Some(&options))
        .await
        .with_context(|| "Failed to challenge Stockfish")
        .map(|challenge| {
            info!(
                "Sent challenge to Stockfish level {}: https://lichess.org/{}",
                level, challenge.id
            );
        })
}

pub async fn abort_games(lichess: Arc<Lichess>) -> anyhow::Result<()> {
    info!("Resigning all live games...");
    for game in lichess
        .get_ongoing_games(50)
        .await
        .with_context(|| "Failed to get ongoing games")?
        .into_iter()
    {
        debug!("Resigning {}...", &game.game_id);
        lichess
            .resign_bot_game(&game.game_id)
            .await
            .with_context(|| format!("Failed to abort game: {}", &game.game_id))?;
    }
    Ok(())
}

pub async fn start_bot(lichess: Arc<Lichess>, config: Config) -> anyhow::Result<()> {
    let mut event_stream = lichess
        .stream_incoming_events()
        .await
        .with_context(|| "Failed to get incoming events")?;

    ascii_art(&config);

    let mut world = World::default();
    while let Some(event) = event_stream.next().await {
        if let Ok(event) = event {
            if let Err(e) =
                process_incoming_event(event, &mut world, lichess.clone(), &config).await
            {
                error!("Failed to process event: {:?}", e);
            }
        } else {
            info!("Stopped processing events from loop.");
            break;
        }
    }

    Ok(())
}

#[rustfmt::skip]
fn ascii_art(config: &Config) {
    let challenge_message = if config.no_accept {
        "I am not accepting challenges, but here is my profile:"
    } else {
        "Send me a challenge on Lichess:"
    };

    info!(r"");
    info!(r"  _/|"                                      );
    info!(r" // o\     Hello, this is {}!"             , &config.username);
    info!(r" || ._)    {}"                             , challenge_message);
    info!(r" //__\     https://lichess.org/@/{}"       , &config.username);
    info!(r" )___("                                     );
    info!(r"");
}
