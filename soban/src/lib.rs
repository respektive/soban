#[macro_use]
extern crate tracing;

mod commands;
mod utils;

use eyre::Result;
use futures::future::BoxFuture;
use irc::client::Sender;
use linkme::distributed_slice;
use matrix_sdk::{room::Joined, ruma::events::room::message::RoomMessageEventContent};
use rosu_v2::Osu;
use std::{
    collections::HashMap,
    iter,
    sync::{Arc, OnceLock},
};

pub enum CommandOrigin<'a> {
    Irc {
        sender: &'a Sender,
        target: &'a String,
    },
    Matrix {
        room: &'a Joined,
    },
}

impl CommandOrigin<'_> {
    pub async fn send(&self, msg: &str) -> Result<()> {
        match self {
            CommandOrigin::Irc { sender, target } => sender.send_privmsg(target, msg)?,
            CommandOrigin::Matrix { room } => {
                room.send(RoomMessageEventContent::text_plain(msg), None)
                    .await?;
            }
        }
        Ok(())
    }
}

pub struct Context {
    pub osu: Osu,
}

type CommandFn = for<'a> fn(Arc<Context>, CommandOrigin<'a>, Args<'a>) -> BoxFuture<'a, Result<()>>;

pub struct Command {
    name: &'static str,
    aliases: &'static [&'static str],
    run: CommandFn,
}

struct Args<'a> {
    msg: &'a str,
    num: Option<u32>,
}

struct Commands(HashMap<&'static str, CommandFn>);

#[distributed_slice]
static COMMANDS_SLICE: [Command] = [..];

static COMMANDS: OnceLock<Commands> = OnceLock::new();

impl Commands {
    pub fn get() -> &'static Self {
        COMMANDS.get_or_init(|| {
            let mut cmds = HashMap::new();

            for cmd in COMMANDS_SLICE {
                let names = iter::once(cmd.name).chain(cmd.aliases.iter().copied());

                for name in names {
                    if cmds.insert(name, cmd.run).is_some() {
                        panic!("command `{name}` has been defined multiple times");
                    }
                }
            }

            Self(cmds)
        })
    }

    pub fn command(&self, name: &str) -> Option<CommandFn> {
        self.0.get(name).copied()
    }
}

pub async fn handle_command(ctx: Arc<Context>, origin: CommandOrigin<'_>, msg: &str) -> Result<()> {
    let Some(stripped_prefix) = msg.strip_prefix('!') else {
        // missing prefix
        return Ok(());
    };

    let (mut next_word, rest) = stripped_prefix
        .split_once(' ')
        .unwrap_or((stripped_prefix, ""));

    let mut num = None;

    if let Some(first_digit_idx) = next_word
        .bytes()
        .rev()
        .position(|byte| byte.is_ascii_alphabetic())
        .filter(|&idx| idx > 0)
    {
        let (front, back) = next_word.split_at(next_word.len() - first_digit_idx);
        next_word = front;
        num = back.parse::<u32>().ok();
    }

    let Some(cmd_fn) = Commands::get().command(next_word) else {
        // unknown command name
        return Ok(());
    };

    info!(name = next_word, num, rest, "Processing command");

    let args = Args { msg: rest, num };

    (cmd_fn)(ctx, origin, args).await
}
