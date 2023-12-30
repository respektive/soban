#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

use eyre::Result;
use futures::StreamExt;
use irc::{
    client::{prelude::Config, Client, Sender},
    proto::{Command, Message},
};
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
    Client as MatrixClient,
};
use rosu_v2::Osu;
use soban::{handle_command, CommandOrigin, Context};
use std::{env, sync::Arc};

struct MatrixConfig {
    homeserver: String,
    username: String,
    password: String,
}

struct BotConfig {
    osu_client_id: u64,
    osu_client_secret: String,
    irc_config: Config,
    matrix_config: MatrixConfig,
}

impl BotConfig {
    pub fn init() -> Result<Self> {
        let osu_client_id = match env::var("OSU_CLIENT_ID") {
            Ok(val) => val.parse()?,
            Err(_) => return Err(eyre!("Missing OSU_CLIENT_ID environment variable")),
        };

        let osu_client_secret = match env::var("OSU_CLIENT_SECRET") {
            Ok(val) => val,
            Err(_) => return Err(eyre!("Missing OSU_CLIENT_SECRET environment variable")),
        };

        let matrix_homeserver = match env::var("MATRIX_HOMESERVER") {
            Ok(val) => val,
            Err(_) => return Err(eyre!("Missing MATRIX_HOMESERVER environment variable")),
        };

        let matrix_user = match env::var("MATRIX_USER") {
            Ok(val) => val,
            Err(_) => return Err(eyre!("Missing MATRIX_USER environment variable")),
        };

        let matrix_password = match env::var("MATRIX_PASSWORD") {
            Ok(val) => val,
            Err(_) => return Err(eyre!("Missing MATRIX_PASSWORD environment variable")),
        };

        let irc_config = Config {
            nickname: Some("soban".to_owned()),
            server: Some("irc.lea.moe".to_owned()),
            channels: vec!["#general".to_owned(), "#osu".to_owned()],
            ..Config::default()
        };

        let matrix_config = MatrixConfig {
            homeserver: matrix_homeserver,
            username: matrix_user,
            password: matrix_password,
        };

        Ok(BotConfig {
            osu_client_id,
            osu_client_secret,
            irc_config,
            matrix_config,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::fmt::init();
    let config = BotConfig::init()?;

    let osu = Osu::new(config.osu_client_id, config.osu_client_secret).await?;
    let context = Arc::new(Context { osu });

    let irc_client = Client::from_config(config.irc_config).await?;
    let matrix_client = MatrixClient::builder()
        .homeserver_url(config.matrix_config.homeserver)
        .build()
        .await?;

    let irc_ctx = Arc::clone(&context);

    let irc_task = tokio::spawn(async move { run_irc_client(irc_ctx, irc_client).await });

    let matrix_task = tokio::spawn(async move {
        run_matrix_client(
            context,
            matrix_client,
            &config.matrix_config.username,
            &config.matrix_config.password,
        )
        .await
    });

    irc_task.await.expect("irc worker panicked")?;
    matrix_task.await.expect("matrix worker panicked")?;

    Ok(())
}

async fn run_matrix_client(
    context: Arc<Context>,
    matrix_client: MatrixClient,
    username: &str,
    password: &str,
) -> Result<()> {
    matrix_client
        .login_username(username, password)
        .send()
        .await?;
    let response = matrix_client
        .sync_once(SyncSettings::default())
        .await
        .unwrap();

    matrix_client.add_event_handler(move |ev: OriginalSyncRoomMessageEvent, room: Room| {
        let ctx = Arc::clone(&context);

        async move { process_matrix_message(ctx, ev, room).await }
    });
    let settings = SyncSettings::default().token(response.next_batch);
    matrix_client.sync(settings).await?;

    Ok(())
}

async fn process_matrix_message(
    context: Arc<Context>,
    event: OriginalSyncRoomMessageEvent,
    room: Room,
) {
    let Room::Joined(ref room) = room else {
        return;
    };
    let MessageType::Text(text_content) = event.content.msgtype else {
        return;
    };

    let origin = CommandOrigin::Matrix { room };

    if let Err(err) = handle_command(context, origin, &text_content.body).await {
        error!(?err, "Failed to handle matrix cmd");
    }
}

async fn run_irc_client(context: Arc<Context>, mut irc_client: Client) -> Result<()> {
    irc_client.identify()?;

    let mut stream = irc_client.stream()?;
    let sender = irc_client.sender();

    while let Some(message) = stream.next().await.transpose()? {
        process_irc_message(context.clone(), &sender, message).await;
    }

    Ok(())
}

async fn process_irc_message(context: Arc<Context>, sender: &Sender, message: Message) {
    if let Command::PRIVMSG(ref target, ref msg) = message.command {
        let origin = CommandOrigin::Irc { sender, target };

        if let Err(err) = handle_command(context, origin, msg).await {
            error!(?err, "Failed to handle irc cmd");
        }
    }
}
