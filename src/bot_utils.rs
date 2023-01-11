use std::collections::HashMap;
use std::error;
use std::sync::Arc;
use serde_yaml;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::macros::check;
use serenity::framework::standard::Reason;
use serenity::model::channel::Message;
use serenity::model::id::{GuildId, RoleId, UserId};
use serenity::prelude::{TypeMapKey};
use tokio::sync::{RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub token: String,
}

impl Default for Credentials{
    fn default() -> Self {
        Credentials{
            token:"<fancy_token>".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BotModes{
    Latex,
    Music,
    Soundboard,
    All,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BotPermission {
    Owner,
    Admin,
    Moderator,
    User,
    None,
}
impl BotPermission {
    fn level(&self) -> u8{
        match self {
            BotPermission::Owner => u8::MAX,
            BotPermission::Admin => 3,
            BotPermission::Moderator => 2,
            BotPermission::User => 1,
            BotPermission::None => 0,
            _ => 0
        }
    }

    fn dominates(&self, perm: &BotPermission) -> bool{
        self.level() >= perm.level()
    }
}

pub struct BotConfig;
impl TypeMapKey for BotConfig {
    type Value = Arc<RwLock<ConfigStruct>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigStruct {
    owner_id: UserId,
    prefix: char,
    auto_reconnect: bool,
    bot_mode: BotModes,
    activity: serenity::model::gateway::ActivityType,
    server_cfgs: HashMap<GuildId, ServerAudioStruct>,
}
impl Default for ConfigStruct{
    fn default() -> Self {
        ConfigStruct{
            owner_id: UserId(0),
            prefix: '!',
            auto_reconnect: false,
            bot_mode: BotModes::Latex,
            activity: serenity::model::gateway::ActivityType::Watching,
            server_cfgs: HashMap::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerAudioStruct {
    volume: u8,
    auto_playlist: bool,
    user_default: bool,
    role_permission: HashMap<RoleId, BotPermission>,
}
impl Default for ServerAudioStruct{
    fn default() -> Self {
        ServerAudioStruct {
            volume: 80,
            auto_playlist: false,
            user_default: false,
            role_permission: HashMap::default(),
        }
    }
}

pub fn write_example_credentials(){
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("bot_credentials.yml")
        .expect("Couldn't open file.");
    serde_yaml::to_writer(f, &Credentials::default()).unwrap();
    println!("Failed to read credential file!\nExample file written instead.");
}

pub fn read_credentials() -> Result<Credentials, Box<dyn error::Error>>{
    let f = std::fs::File::open("./bot_credentials.yml")?;
    Ok::<Credentials, _>(serde_yaml::from_reader(f)?)
}

pub fn write_example_config(){
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("bot_config.yml")
        .expect("Couldn't open file.");
    serde_yaml::to_writer(f, &ConfigStruct::default()).unwrap();
    println!("Failed to read config file!\nExample file written instead.");
}

pub fn read_config() -> Result<ConfigStruct, Box<dyn error::Error>>{
    let f = std::fs::File::open("./bot_config.yml")?;
    Ok::<ConfigStruct, _>(serde_yaml::from_reader(f)?)
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
pub fn check_msg(result: serenity::Result<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub async fn user_permission(ctx: &Context, msg: &Message) -> Result<BotPermission, Reason>{
    let data = ctx.data.read().await;

    let bot_config = match data.get::<BotConfig>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the bot config").await.unwrap();
            return Err(Reason::User("Bot config failed!".to_string()));
        },
    };
    let bot_config = bot_config.read().await;

    let owner_id = bot_config.owner_id;
    let author_id = msg.author.id;

    if author_id == owner_id{
        return Ok(BotPermission::Owner);
    }


    let mut ret_perm = BotPermission::None;

    if let Some(guild_id) = msg.guild_id {
        if let Some(guild_cfg) = bot_config.server_cfgs.get(&guild_id){
            // check if all users default to User permission
            if guild_cfg.user_default{
                ret_perm = BotPermission::User;
            }
            // check if user has a role with sufficient permission
            if let Some(mem) = &msg.member{
                for role in &mem.roles{
                    if let Some(perm) = guild_cfg.role_permission.get(&role){
                        if perm.dominates(&ret_perm){
                            ret_perm = *perm;
                        }
                    }
                }
            }
        }else{
            return Err(Reason::User("Server config doesnt exist!".to_string()))
        }
    }

    return Ok(ret_perm);
}

async fn verify_permission(ctx: &Context, msg: &Message, command_permission: BotPermission) -> Result<(), Reason>{

    if user_permission(ctx, msg).await?.dominates(&command_permission){
        return Ok(());
    }

    Err(Reason::User("Insufficient Permission!".to_string()))
}

#[check]
async fn verify_owner(ctx: &Context, msg: &Message) -> Result<(), Reason>{
    verify_permission(ctx, msg, BotPermission::Owner).await
}

#[check]
async fn verify_admin(ctx: &Context, msg: &Message) -> Result<(), Reason>{
    verify_permission(ctx, msg, BotPermission::Admin).await
}

#[check]
async fn verify_moderator(ctx: &Context, msg: &Message) -> Result<(), Reason>{
    verify_permission(ctx, msg, BotPermission::Moderator).await
}

#[check]
async fn verify_user(ctx: &Context, msg: &Message) -> Result<(), Reason>{
    verify_permission(ctx, msg, BotPermission::User).await
}
