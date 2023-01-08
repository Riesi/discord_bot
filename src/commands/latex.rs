use std::borrow::Cow;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::AttachmentType::Bytes;
use serenity::model::channel::Message;

use crate::latex_utils;

#[group]
//#[summary = "Latex commands"]
#[commands(math, tex)]
pub struct Latex;

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
pub async fn math(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
pub async fn tex(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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