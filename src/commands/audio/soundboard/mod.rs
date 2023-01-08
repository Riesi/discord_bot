use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

use crate::bot_utils::check_msg;

#[group]
//#[summary = "Soundboard commands"]
#[commands(sb)]
pub struct Soundboard;

#[command]
#[only_in(guilds)]
pub async fn sb(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let path = match args.single::<String>() {
        Ok(path) => path,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a path to a video or audio").await);

            return Ok(());
        },
    };

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ffmpeg(&path).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            },
        };
        handler.play_source(source);

        check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
    }

    Ok(())
}