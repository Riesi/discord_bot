use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

use crate::bot_utils::check_msg;
use crate::commands::audio::Player;

#[group]
//#[summary = "Music commands"]
#[commands(play, resume, stop, pause)]
pub struct Music;

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await);

            return Ok(());
        },
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL").await);

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            },
        };
        let track_handler = handler.play_source(source);
        let mut data = ctx.data.write().await;
        let players = data.get_mut::<Player>().expect("Expected Player in TypeMap.");
        let handle_map = players.insert(guild_id.to_string(),track_handler);

        check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut data = ctx.data.write().await;
        let players = data.get_mut::<Player>().expect("Expected Player in TypeMap.");
        if let Some(track_handler) = players.remove(&guild_id.to_string()){
            track_handler.stop().expect("Can not stop!");
            check_msg(msg.channel_id.say(&ctx.http, "Stopping song").await);
        }else{
            check_msg(msg.channel_id.say(&ctx.http, "No song to stop").await);
        }
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to stop").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let players = data.get::<Player>().expect("Expected Player in TypeMap.");
    if let Some(track_handler) = players.get(&guild_id.to_string()){
        track_handler.pause().expect("Can not pause!");
    }else{
        check_msg(msg.channel_id.say(&ctx.http, "No song to pause").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let players = data.get::<Player>().expect("Expected Player in TypeMap.");
    if let Some(track_handler) = players.get(&guild_id.to_string()){
        track_handler.play().expect("Can not resume!");
    }else{
        check_msg(msg.channel_id.say(&ctx.http, "No song to resume").await);
    }

    Ok(())
}