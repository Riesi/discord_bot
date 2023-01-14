use std::sync::Arc;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

use crate::bot_utils::*;

#[group]
#[commands(latency,whoami,whois)]
pub struct General;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[command]
#[checks(verify_user)]
pub async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager").await?;

            return Ok(());
        },
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        },
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.latency)).await?;

    Ok(())
}

#[command]
#[checks(verify_user)]
pub async fn whoami(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, &format!("You are {:?}", user_permission(ctx, msg, msg.author.id).await)).await.expect("User has no permission?");
    Ok(())
}

#[command]
#[checks(verify_user)]
pub async fn whois(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let choosen_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "No id provided").await);
            return Ok(());
        },
    };
    msg.reply(ctx, &format!("Given id is {:?}", user_permission(ctx, msg, choosen_id).await)).await.expect("User has no permission?");
    Ok(())
}