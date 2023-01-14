use std::collections::HashMap;
use songbird::tracks::TrackHandle;

use serenity::client::Context;
use serenity::prelude::TypeMapKey;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;
use serenity::model::id::GuildId;

use crate::bot_utils::*;

#[group]
//#[summary = "Audio commands"]
#[commands(deafen, join, leave, mute, undeafen, unmute, set_volume)]
pub struct Audio;

pub mod music;
pub mod soundboard;

pub struct Player;
impl TypeMapKey for Player {
    type Value = HashMap<GuildId, TrackHandle>;
}

#[command]
#[only_in(guilds)]
#[checks(verify_moderator)]
pub async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        check_msg(msg.channel_id.say(&ctx.http, "Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Deafened").await);
    }

    Ok(())
}

pub async fn join_channel(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(connect)]
#[checks(verify_user)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    join_channel(ctx, msg).await
}

#[command]
#[only_in(guilds)]
#[aliases(disconnect)]
#[checks(verify_user)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(verify_moderator)]
pub async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_msg(msg.channel_id.say(&ctx.http, "Already muted").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Now muted").await);
    }

    Ok(())
}



#[command]
#[only_in(guilds)]
#[checks(verify_moderator)]
pub async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Undeafened").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to undeafen in").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(verify_moderator)]
pub async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Unmuted").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to unmute in").await);
    }

    Ok(())
}

async fn get_volume(ctx: &Context, guild: GuildId) -> u8{
    let data = ctx.data.write().await;

    let bot_config = match data.get::<BotConfig>() {
        Some(v) => v,
        None => {
            return 10;
        },
    };

    let mut bot_config = bot_config.write().await;
    bot_config.get_guild_volume(guild)
}

#[command]
#[only_in(guilds)]
#[description("Sets the volume of the bot")]
#[usage("Values from 10..100 are allowed.")]
#[checks(verify_moderator)]
pub async fn set_volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;

    let bot_config = match data.get::<BotConfig>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the bot config!").await.unwrap();
            return Ok(());
        },
    };
    let mut bot_config = bot_config.write().await;

    let volume = match args.single::<u8>() {
        Ok(vol) => vol,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "No volume provided!").await);
            return Ok(());
        },
    };
    if let Some(guild) = msg.guild_id{
        bot_config.set_guild_volume(guild, volume);
        let players = data.get::<Player>().expect("Expected Player in TypeMap.");
        if let Some(track_handler) = players.get(&guild){
            track_handler.set_volume((volume as f32)/100f32).expect("Can not set volume!");
        }
    }
    write_config(&bot_config).expect("Config could not be written!");

    Ok(())
}