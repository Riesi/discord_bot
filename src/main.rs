use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::buckets::LimitedFor;
use serenity::framework::standard::{Args, CommandGroup, CommandResult, DispatchError, help_commands, HelpOptions};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::{Channel, Message};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::framework::standard::macros::{command, hook, help, group};
use serenity::model::channel::AttachmentType::Bytes;
use serenity::model::id::UserId;

mod bot_utils;
mod latex_utils;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

#[group]
#[commands(latency, tex, math)]
struct General;

#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server owners"]
#[commands(slow_mode)]
struct Owner;

#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) =
            msg.channel_id.edit(&ctx.http, |c| c.rate_limit_per_user(slow_mode_rate_seconds)).await
        {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!("Failed to set slow mode to `{}` seconds.", slow_mode_rate_seconds)
        } else {
            format!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds)
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache) {
        let slow_mode_rate = channel.rate_limit_per_user.unwrap_or(0);
        format!("Current slow mode rate is `{}` seconds.", slow_mode_rate)
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
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

async fn latex_handling(ctx: &Context, msg: &Message, tex_string: String){
    if let Ok(image) = tokio::task::spawn_blocking(move || {
            latex_utils::latex_tex_png(&tex_string)
    }).await.expect("LaTeX future didn't hold!"){
        msg.channel_id.send_message(&ctx,|m| {
            // Reply to the given message
            //m.reference_message(&msg);
            // Attach image
            m.add_file(Bytes {
                    data: Cow::from(image.as_slice()),
                    filename: "image.png".to_string(),
                });
            m
        })
        .await.expect("Feedback failed!");
    }else{
        msg.reply(ctx, &format!("Invalid LaTeX syntax!")).await.expect("Feedback failed!");
    }
}

#[command]
async fn math(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(message_string) => {
            latex_handling(ctx, msg,("$\\displaystyle\n".to_owned() + &message_string + "$").to_string()).await;
        },
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.").await?;
        },
    };
    Ok(())
}

#[command]
async fn tex(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(message_string) => {
            latex_handling(ctx, msg,(message_string.to_owned()).to_string()).await;
        },
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.").await?;
        },
    };
    return Ok(());
}

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "-"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    if let Ok(cred) = bot_utils::read_credentials(){
        let http = Http::new(&cred.token);

        // We will fetch your bot's owners and id
        let (owners, bot_id) = match http.get_current_application_info().await {
            Ok(info) => {
                let mut owners = HashSet::new();
                if let Some(team) = info.team {
                    owners.insert(team.owner_user_id);
                } else {
                    owners.insert(info.owner.id);
                }
                match http.get_current_user().await {
                    Ok(bot_id) => (owners, bot_id.id),
                    Err(why) => panic!("Could not access the bot id: {:?}", why),
                }
            },
            Err(why) => panic!("Could not access application info: {:?}", why),
        };

        let framework = StandardFramework::new()
            .configure(|c| c
                       .with_whitespace(true)
                       .on_mention(Some(bot_id))
                       .prefix("!")
                       .delimiters(vec![", ", ","])
                       .owners(owners))
            .before(before) //before command execution
            .after(after) //after command execution
            .unrecognised_command(unknown_command)
            .on_dispatch_error(dispatch_error)
            .bucket("emoji", |b| b.delay(5)).await
            .bucket("complicated", |b| b.limit(2).time_span(30).delay(5)
            .limit_for(LimitedFor::Channel)
            .await_ratelimits(1)
            .delay_action(delay_action)).await
            .help(&MY_HELP)
            .group(&GENERAL_GROUP)
            .group(&OWNER_GROUP);

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client =
            Client::builder(&cred.token, intents)
                .event_handler(Handler)
                .framework(framework)
                .type_map_insert::<CommandCounter>(HashMap::default())
                .await.expect("Err creating client");
        {
            let mut data = client.data.write().await;
            data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        }

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    }else{
        bot_utils::write_example_credentials();
    }
}
