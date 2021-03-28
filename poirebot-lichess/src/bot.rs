use std::collections::HashMap;
use std::sync::Arc;

use crate::licorice::client::Lichess;
use crate::licorice::models::board::{BoardState, Challenge, Challengee, Event, GameFull, GameID};
use crate::licorice::models::user::User;
use anyhow::Context;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio_stream::StreamExt;

use crate::licorice::models::game::Player;
use poirebot::game::pieces::Color;
use poirebot::game::{Board, Move};
use poirebot::genius::Brain;
use std::time::SystemTime;

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
    /// When a move comes through. The boolean is for whether the game is over.
    Move(Move, Color, bool),
    /// (re)Set the board (initial FEN, UCI moves, own color)
    SetBoard(String, Vec<Move>, Color),
    /// Handle when someone requests a draw
    DrawOffer(Color),
}

/// Configures the bot.
#[derive(Debug, Clone)]
pub struct Config {
    /// The Lichess.org username of the bot.
    pub username: String,
    /// If true, does not accept any incoming challenge.
    pub no_accept: bool,
    /// Whether to send rematch after each game.
    pub rematch: bool,
    /// The Stockfish level to play against (if applicable).
    pub stockfish: u8,
    /// The maximum stockfish level to play against (if applicable).
    pub stockfish_max: u8,
    /// Whether to only accept from users the bot account follows.
    pub following_only: bool,
}

async fn find_and_send_move(
    lichess: Arc<Lichess>,
    game_id: &str,
    brain: &mut Brain,
) -> anyhow::Result<()> {
    let (sensor, recv) = oneshot::channel::<Option<Move>>();
    let current_time = SystemTime::now();
    brain.choose_move(sensor);

    let m = recv
        .await
        .with_context(|| "communication failure")?
        .with_context(|| "ran out of moves")?;

    let duration = current_time.elapsed().unwrap();
    lichess
        .write_in_bot_chat(
            game_id,
            "player",
            format!("Move generation took {} microseconds", duration.as_micros()).as_str(),
        )
        .await
        .unwrap_or(());

    lichess
        .make_a_bot_move(game_id, m.to_pure_notation().as_str(), false)
        .await
        .with_context(|| "Failed to dispatch move to Lichess")
}

/// Task that handles new game state messages.
async fn message_loop(
    game_id: GameID,
    recv: &mut UnboundedReceiver<Message>,
    lichess: Arc<Lichess>,
    config: &Config,
) {
    let mut brain = Brain::new(Board::default(), Color::White); // Temporary value

    while let Some(message) = recv.recv().await {
        debug!("({}) message loop: {:?}", &game_id.id, message);
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
            }
            Message::Abort => {
                info!("Game/Challenge aborted: {}", game_id.id);
                break;
            }
            Message::BoardChat(username, message) => {
                info!("({})\t\t{}\t\t{}", game_id.id, username, message);
                if message == ".version" {
                    lichess
                        .write_in_bot_chat(
                            &game_id.id,
                            "player",
                            format!("Poirebot version: {}", clap::crate_version!()).as_str(),
                        )
                        .await
                        .unwrap_or(());
                }
            }
            Message::Move(m, color, game_over) => {
                let bot_move = color == brain.color;
                if bot_move {
                    if brain.last_move == Some(m) {
                        debug!("Ignored repeated bot move: {}", m.to_pure_notation());
                        continue;
                    }
                    debug!("Bot moved: {}", m.to_pure_notation());
                    brain.own_move(m);
                } else {
                    if brain.opponent_last_move == Some(m) {
                        debug!("Ignored repeated opponent move: {}", m.to_pure_notation());
                        continue;
                    }
                    debug!("Opponent ({}) moved: {}", game_id.id, m.to_pure_notation());
                    brain.opponent_move(m);

                    if game_over {
                        break;
                    }

                    if let Err(e) =
                        find_and_send_move(lichess.clone(), &game_id.id, &mut brain).await
                    {
                        error!("{:?}", e);
                        lichess.resign_bot_game(&game_id.id).await.unwrap_or(());
                        break;
                    }
                }
            }
            Message::SetBoard(fen, moves, own_color) => {
                let mut board = Board::from_fen(&fen).expect("lichess sent invalid fen");
                moves.iter().for_each(|m| board.apply_move(m.to_owned()));
                brain = Brain::new(board, own_color);

                let bots_turn = match own_color {
                    Color::Black => moves.len() % 2 == 1,
                    Color::White => moves.len() % 2 == 0,
                };

                let (last_move, opp_last_move) = if bots_turn {
                    (
                        moves.get(moves.len().wrapping_sub(2)).map(Move::to_owned),
                        moves.last().map(Move::to_owned),
                    )
                } else {
                    (
                        moves.last().map(Move::to_owned),
                        moves.get(moves.len().wrapping_sub(2)).map(Move::to_owned),
                    )
                };
                brain.last_move = last_move;
                brain.opponent_last_move = opp_last_move;

                if bots_turn {
                    if let Err(e) =
                        find_and_send_move(lichess.clone(), &game_id.id, &mut brain).await
                    {
                        error!("{:?}", e);
                        lichess.resign_bot_game(&game_id.id).await.unwrap_or(());
                        break;
                    }
                }
            }
            Message::DrawOffer(_) => {
                // Ignore draw offers right now
                // Note: this gets declined automatically when the other player/bot moves
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
    let challenger = challenge.challenger.as_ref().unwrap();

    // TODO: Determine acceptable modes/time.
    let accept = {
        if config.no_accept {
            false
        } else if config.following_only
            && !is_following(lichess.clone(), &config.username, &challenger.username)
                .await
                .unwrap_or(false)
        {
            debug!(
                "Declining challenge by {} because they are not followed",
                &challenger.username
            );
            false
        } else {
            true
        }
    };

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

async fn is_following(
    lichess: Arc<Lichess>,
    bot_username: &str,
    username: &str,
) -> anyhow::Result<bool> {
    let mut stream = lichess
        .get_followings(bot_username)
        .await
        .with_context(|| "Failed to get followings")?;
    while let Some(user) = stream.next().await {
        if let Ok(user) = user {
            if user.username == username {
                return Ok(true);
            } else {
                continue;
            }
        }
    }
    Ok(false)
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

    let game_id = GameID { id: game_id };
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

        match event_stream {
            Ok(mut event_stream) => {
                while let Some(event) = event_stream.next().await {
                    if let Ok(event) = event {
                        dispatch_board_event(&sender, &id, event, &config_b).await;
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
                    .unwrap_or(());
            }
        }
        BoardState::GameFull(state) => {
            if state.state.status == "started" {
                let is_white = is_bot_white(&state, &config.username);
                let color = if is_white { Color::White } else { Color::Black };

                let initial_fen = state.initial_fen;
                let moves = state
                    .state
                    .moves
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .map(Move::from_pure_notation)
                    .collect::<Vec<Move>>();

                sender
                    .send(Message::SetBoard(initial_fen, moves, color))
                    .unwrap_or(());
            } else {
                warn!("Unhandled board status: {}", state.state.status);
            }
        }
        BoardState::GameState(state) => {
            if state.status == "started" {
                if state.bdraw {
                    sender.send(Message::DrawOffer(Color::Black)).unwrap_or(());
                } else if state.wdraw {
                    sender.send(Message::DrawOffer(Color::White)).unwrap_or(());
                }

                let moves = state
                    .moves
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .map(Move::from_pure_notation)
                    .collect::<Vec<Move>>();

                let last_move_color = if moves.len() % 2 == 1 {
                    Color::White
                } else {
                    Color::Black
                };

                let last_move = moves.last().unwrap().to_owned();

                // TODO: Handle draw
                let game_over = state.winner.is_some();

                sender
                    .send(Message::Move(last_move, last_move_color, game_over))
                    .unwrap_or(());
            } else {
                warn!("Unhandled board status: {}", state.status);
            }
        }
    }
}

/// Dispatches the 'Abort' message to the game, closing it.
async fn abort_task(game_id: &str, world: &mut World) {
    if let Some(sender) = world.games.get(game_id) {
        sender.send(Message::Abort).unwrap_or(());
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

        Event::ChallengeCanceled { challenge } => {
            abort_task(&challenge.id, world).await;
            Ok(())
        }

        Event::ChallengeDeclined { challenge } => {
            abort_task(&challenge.id, world).await;
            Ok(())
        }

        Event::GameFinish { game } => {
            abort_task(&game.id, world).await;
            if config.rematch {
                send_rematch(&config, lichess.clone(), &game.id)
                    .await
                    .with_context(|| "Failed to send rematch")?;
            }
            Ok(())
        }

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

pub async fn send_rematch(
    config: &Config,
    lichess: Arc<Lichess>,
    game_id: &str,
) -> anyhow::Result<()> {
    let game = lichess
        .export_one_game_json(&game_id, None)
        .await
        .with_context(|| "Failed to fetch game")?;
    // Determine opponent
    let opponent = match &game.players.white {
        Player::Entity(e) => {
            if e.user.as_ref().unwrap().username == config.username {
                game.players.black
            } else {
                game.players.white
            }
        }
        Player::StockFish(_) => game.players.white,
    };
    match opponent {
        Player::Entity(human) => {
            send_user_challenge(lichess.clone(), human.user.unwrap().username).await
        }
        Player::StockFish(stockfish) => {
            send_stockfish_challenge(
                lichess.clone(),
                (stockfish.ai_level + 1).clamp(stockfish.ai_level, config.stockfish_max),
            )
            .await
        }
    }
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

pub async fn upgrade_bot_account(lichess: Arc<Lichess>, user: &User) -> anyhow::Result<()> {
    warn!("Upgrading account {} to a BOT account...", user.username);
    lichess
        .upgrade_to_bot_account()
        .await
        .with_context(|| "Upgrade failed")
}
