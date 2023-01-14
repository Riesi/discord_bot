use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::buckets::LimitedFor;
use serenity::framework::standard::{Args, CommandGroup, CommandResult, DispatchError, help_commands, HelpOptions};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::{Message};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::framework::standard::macros::{hook, help};
use serenity::model::id::UserId;
use songbird::SerenityInit;

mod bot_utils;
mod latex_utils;
mod commands;
mod entity_id;

use commands::audio::Player;
use crate::bot_utils::BotConfig;
use crate::commands::general::ShardManagerContainer;

struct CommandCounter;
impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
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
    tracing_subscriber::fmt::init();

    if let Ok(cred) = bot_utils::read_credentials(){
        if let Ok(mut cfg) = bot_utils::read_config(){
            let http = Http::new(&cred.token);
            // We will fetch your bot's owners and id
            let (owners, bot_id, bot_guilds) = match http.get_current_application_info().await {
                Ok(info) => {
                    let mut owners = HashSet::new();
                    if let Some(team) = info.team {
                        owners.insert(team.owner_user_id);
                    } else {
                        owners.insert(info.owner.id);
                    }

                    match http.get_current_user().await {
                        Ok(bot_id) => {
                            let bot_guilds = bot_id.guilds(http).await
                                                                 .unwrap_or(Vec::default());
                            (owners, bot_id.id, bot_guilds)
                        },
                        Err(why) => panic!("Could not access the bot id: {:?}", why),
                    }
                },
                Err(why) => panic!("Could not access application info: {:?}", why),
            };

            for guild in bot_guilds{
                cfg.init_server(guild.id);
            }

            println!("Config {:#?}", cfg);
            bot_utils::write_config(&cfg).expect("Config could not be written!");

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
                .group(&commands::general::GENERAL_GROUP)
                .group(&commands::latex::LATEX_GROUP)
                .group(&commands::audio::AUDIO_GROUP)
                .group(&commands::audio::music::MUSIC_GROUP)
                .group(&commands::moderation::MODERATION_GROUP)
                .group(&commands::audio::soundboard::SOUNDBOARD_GROUP)
                .group(&commands::owner::OWNER_GROUP);

            let intents = GatewayIntents::non_privileged()
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT;

            let mut client =
                Client::builder(&cred.token, intents)
                    .event_handler(Handler)
                    .framework(framework)
                    .register_songbird()
                    .type_map_insert::<CommandCounter>(HashMap::default())
                    .type_map_insert::<Player>(HashMap::default())
                    .type_map_insert::<BotConfig>(Arc::new(RwLock::new(cfg)))
                    .await.expect("Err creating client");
            {
                let mut data = client.data.write().await;
                data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
            }

            if let Err(why) = client.start().await {
                println!("Client error: {:?}", why);
            }
        }else{
            bot_utils::write_example_config();
        }
    }else{
        bot_utils::write_example_credentials();
    }
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
