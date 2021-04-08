# poirebot

A chess engine and bot written in Rust.

[![Build status](https://github.com/aramperes/poirebot/actions/workflows/build.yml/badge.svg)](https://github.com/aramperes/poirebot/actions)
[![Crates.io](https://img.shields.io/crates/v/poirebot.svg)](https://crates.io/crates/poirebot)
[![Crates.io](https://img.shields.io/github/v/tag/aramperes/poirebot?label=release)](https://github.com/aramperes/poirebot/releases/latest)

## Playing against poirebot

The bot is occasionally up on Lichess.org with the account [@poirebot](https://lichess.org/@/poirebot).

Until the bot is live 24/7, you may want to run it locally to try it out. You should create a new Lichess.org that will
then become a *BOT* account.

1. Create a new [Lichess.org](https://lichess.org) account for the bot
2. Generate a new Personal Access Token (PAT) by going to https://lichess.org/account/oauth/token/create
3. Give it a description, and select all options **except:** *Read Preferences, Write preferences, Read email address*
4. Store the token in the `LICHESS_TOKEN` environment variable (or you can also use the `--token` flag later)
5. You can now install and run the bot

**Download latest release (does not require Rust)**: https://github.com/aramperes/poirebot/releases/latest

**Or to build and install latest release using Cargo (Rust 1.50+):**

```
rustup update stable
cargo install poirebot-lichess
```

**Or to build bleeding edge (master branch) instead (Rust 1.50+):**

```
rustup update stable
git clone https://github.com/aramperes/poirebot.git
cd poirebot
cargo build --release
# (The poirebot-lichess binary will be in ./target/release)
```

### Running

The `poirebot-lichess upgrade-account` command is only required for the first run (it converts the Lichess account into
a *BOT* account).

```
poirebot-lichess upgrade-account
poirebot-lichess start
```

Use `poirebot-lichess start --help` for a list of flags that can be used when running the bot.

## Architecture

poirebot keeps track of board state in a collection of 64-bit [Bitboards](https://www.chessprogramming.org/Bitboards).
Specifically, the board state is stored in a collection of Bitboards for each side (color):

* pawns
* knights
* bishops
* queens
* king
* unmoved rooks
* en passant target square

It also stores whether the king has moved (bool).

In addition, after the mutation of a side (color) is "committed", it generates "inherited" bitboards for move-generation
purposes:

* pieces (the union of pawns, knights, bishops, queens, and king)
* attacks (squares threatened by any piece of that color)

## Move Generation

poirebot does not have a *functioning brain* right now. The current goal is to be able to generate all potential moves,
and later use some flavor of the MiniMax algorithm.

## Dependencies

poirebot depends on Rust stable **1.50+**, or nightly, as it uses the `<number>::clamp` function.

* anyhow: for error handling
* tokio, tokio-stream: async runtime
* rayon: parallelism library for expensive board operations on the CPU
* licorice: Lichess.org API client ([forked](https://gitlab.com/momothereal/licorice/-/tree/poirebot-patches))
* clap: CLI parsing
* rand: PRNG

## Inspiration

This bot was made for a friendly competition with [Jeff](https://github.com/BorysSerbyn/Jeff-bot). As a fairness rule,
neither bot uses a chess library to manage board state and implement movement rules.

## License

MIT, see LICENSE.
