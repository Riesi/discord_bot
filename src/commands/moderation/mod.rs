use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use crate::bot_utils::*;

#[group]
#[commands(make_admin,make_moderator,make_user,demote,set_user_default)]
pub struct Moderation;

async fn make_perm(ctx: &Context, msg: &Message, mut args: Args, perm: BotPermission) -> CommandResult {
    let mut data = ctx.data.write().await;

    let bot_config = match data.get_mut::<BotConfig>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the bot config!").await.unwrap();
            return Ok(());
        },
    };
    let mut bot_config = bot_config.write().await;

    let choosen_id = match args.single::<u64>() {
        Ok(id) => id,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "No id provided!").await);
            return Ok(());
        },
    };
    if let Some(guild) = msg.guild_id{
        bot_config.insert_entity_guild( guild, choosen_id, perm);
    }
    write_config(&bot_config).expect("Config could not be written!");
    Ok(())
}

#[command]
#[only_in(guilds)]
#[description("Makes given role id an admin on the given server")]
#[checks(verify_owner)]
pub async fn make_admin(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    make_perm(ctx, msg, args, BotPermission::Admin).await
}

#[command]
#[only_in(guilds)]
#[description("Makes given role id an moderator on the given server")]
#[checks(verify_admin)]
pub async fn make_moderator(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    make_perm(ctx, msg, args, BotPermission::Moderator).await
}

#[command]
#[only_in(guilds)]
#[description("Makes given role id an user on the given server")]
#[checks(verify_moderator)]
pub async fn make_user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    make_perm(ctx, msg, args, BotPermission::User).await
}

#[command]
#[only_in(guilds)]
#[description("Makes given role id an user on the given server")]
#[checks(verify_moderator)]
pub async fn demote(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    make_perm(ctx, msg, args, BotPermission::None).await
}

#[command]
#[only_in(guilds)]
#[description("Configures if all users default to the User permission")]
#[usage("Values true/false are allowed")]
#[checks(verify_admin)]
pub async fn set_user_default(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;

    let bot_config = match data.get::<BotConfig>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the bot config!").await.unwrap();
            return Ok(());
        },
    };
    let mut bot_config = bot_config.write().await;

    let setting = match args.single::<bool>() {
        Ok(value) => value,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "No boolean provided!").await);
            return Ok(());
        },
    };
    if let Some(guild) = msg.guild_id{
        bot_config.set_guild_user_default(guild, setting);
    }
    write_config(&bot_config).expect("Config could not be written!");

    Ok(())
}