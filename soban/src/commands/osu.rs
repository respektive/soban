use std::sync::Arc;

use eyre::Result;

use soban_macros::command;

use crate::{
    utils::osu::{handle_osu, handle_recent, parse_user_id, require_user_id, RecentArgs},
    Args, CommandOrigin, Context,
};

#[command]
async fn osu(ctx: Arc<Context>, origin: CommandOrigin<'_>, args: Args<'_>) -> Result<()> {
    let Some(user_id) = parse_user_id(args.msg) else {
        return require_user_id(origin).await;
    };

    handle_osu(ctx, origin, user_id).await?;

    Ok(())
}

#[command(aliases("rs"))]
async fn recent(ctx: Arc<Context>, origin: CommandOrigin<'_>, args: Args<'_>) -> Result<()> {
    let Some(user) = parse_user_id(args.msg) else {
        return require_user_id(origin).await;
    };
    let recent_args = RecentArgs {
        user,
        idx: args.num,
        include_fails: true,
    };
    handle_recent(ctx, origin, recent_args).await?;

    Ok(())
}

#[command(aliases("rp"))]
async fn recentpass(ctx: Arc<Context>, origin: CommandOrigin<'_>, args: Args<'_>) -> Result<()> {
    let Some(user) = parse_user_id(args.msg) else {
        return require_user_id(origin).await;
    };
    let recent_args = RecentArgs {
        user,
        idx: args.num,
        include_fails: false,
    };
    handle_recent(ctx, origin, recent_args).await?;

    Ok(())
}
