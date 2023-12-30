use std::sync::Arc;

use eyre::Result;
use soban_macros::command;

use crate::{Args, CommandOrigin, Context};

#[command(aliases("p"))]
async fn ping(_ctx: Arc<Context>, origin: CommandOrigin<'_>, _args: Args<'_>) -> Result<()> {
    origin.send("pong!").await?;

    Ok(())
}
