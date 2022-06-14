use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::{
        channel::Message,
        id::UserId,
    },
    utils::Colour,
};
use std::time::Instant;

#[command]
async fn avatar(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    let face = match args.single::<UserId>() {
        Ok(user_id) => user_id.to_user(ctx).await?.face(),
        Err(_) => msg.author.face(),
    };

    msg.channel_id.say(&ctx.http, face).await?;

    Ok(())
}

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    let before = Instant::now();
    let mut m = msg.channel_id.say(&ctx.http, "pong!").await?;
    let after = Instant::now();

    let content = m.content.clone();
    m.edit(ctx, |m| m.content(format!("{} - {}ms", content, (after - before).as_millis()))).await?;

    Ok(())
}

#[command]
pub async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    if args.is_empty() {
        msg.reply(&ctx, "You need to mention someone to ban!").await?;
    }

    let user_id = u64::from(args.single::<UserId>().unwrap());

    let mut reason = args.rest();

    if reason.is_empty() {
        reason = "No reason";
    }

    match msg.guild_id {
        Some(guild_id) => {
            let guild_id = u64::from(guild_id);
            ctx.http.ban_user(guild_id, user_id, 7, reason).await.unwrap();
            println!("{}", reason);
        },
        _ => {
            msg.reply(&ctx.http, "Cannot get guild id").await.unwrap();

            ()
        },
    }

    Ok(())
}

#[command]
pub async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    if args.is_empty() {
        msg.reply(&ctx, "You need to mention someone to ban!").await?;
    }

    let user_id = u64::from(args.single::<UserId>().unwrap());

    match msg.guild_id {
        Some(guild_id) => {
            let guild_id = u64::from(guild_id);
            ctx.http.kick_member(guild_id, user_id).await.unwrap();
        },
        _ => {
            msg.reply(&ctx.http, "Cannot get guild id").await.unwrap();

            ()
        },
    }

    Ok(())
}

#[command]
pub async fn help(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    if args.is_empty() {
        msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content("")
                    .embed(|e| {
                        e.title("All HeavensBot's commands")
                            .description("This is a description")
                            .fields(vec![
                                ("!help [mod | general]", "Show this message", false),
                                ("!ban", "Ban mentioned member", false),
                                ("!kick", "Kick mentioned member", false),
                                ("!mute", "Add the muted role to the mentioned member", false),
                                ("!unmute", "Remove the muted role to the mentioned member", false),
                                ("!ping", "Show the bot's ping", false),
                                ("!say <message>", "Say the message", false),
                            ])
                            .footer(|f| f.text("©️ HeavensBot 2022"))
                            .timestamp(chrono::Utc::now())
                            .color(Colour::RED)
                    })
            })
            .await
            .expect("Failed to send message");
    } else if args.rest() == "mod" {
        msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content("")
                    .embed(|e| {
                        e.title("All HeavensBot's moderation commands")
                            .description("This is a description")
                            .fields(vec![
                                ("!ban <@member> [reason]", "Ban mentioned member", false),
                                ("!kick <@member>", "Kick mentioned member", false),
                                ("!mute <@member>", "Add the muted role to the mentioned member", false),
                                ("!unmute <@member>", "Remove the muted role to the mentioned member", false),
                            ])
                            .footer(|f| f.text("©️ HeavensBot 2022"))
                            .timestamp(chrono::Utc::now())
                            .color(Colour::RED)
                    })
            })
            .await
            .expect("Failed to send message");
    } else if args.rest() == "general" {
        msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content("")
                    .embed(|e| {
                        e.title("All HeavensBot's general commands")
                            .description("This is a description")
                            .fields(vec![
                                ("!help [mod | general]", "Show this message", false),
                                ("!ping", "Show the bot's ping", false),
                                ("!say <message>", "Say the message", false),
                            ])
                            .footer(|f| f.text("©️ HeavensBot 2022"))
                            .timestamp(chrono::Utc::now())
                            .color(Colour::RED)
                    })
            })
            .await
            .expect("Failed to send message");
    } else {
        msg.channel_id.say(&ctx.http, "This category doesn't exists.").await.expect("Failed to send message");
    }

    Ok(())
}

#[command]
pub async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    if args.is_empty() {
        msg.reply(&ctx, "You need to mention someone to mute!").await?;
    } else {
        // Add role to the mentioned member
        let user_id = u64::from(args.single::<UserId>().unwrap());

        let guild_id = msg.guild_id.expect("Failed to get guild id");

        let role_id_map = guild_id.roles(&ctx.http.as_ref()).await.unwrap();

        let role_id_iter = role_id_map.iter();

        let mut role_id = None;

        for (id, role) in role_id_iter {
            if role.name == "Muted" {
                role_id = Some(id);
            }
        }

        ctx.http.add_member_role(u64::from(guild_id), user_id, u64::from(*role_id.expect("Failed to get role"))).await.expect("Failed to add role");

        msg.reply(&ctx, format!("<@!{}> has been muted", user_id)).await?;
    }

    Ok(())
}

#[command]
pub async fn unmute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    if args.is_empty() {
        msg.reply(&ctx, "You need to mention someone to unmute!").await?;
    } else {
        let user_id = u64::from(args.single::<UserId>().unwrap());

        let guild_id = msg.guild_id.expect("Failed to get guild id");

        let role_id_map = guild_id.roles(&ctx.http.as_ref()).await.unwrap();

        let role_id_iter = role_id_map.iter();

        let mut role_id = None;

        for (id, role) in role_id_iter {
            if role.name == "Muted" {
                role_id = Some(id);
            }
        }

        ctx.http.remove_member_role(u64::from(guild_id), user_id, u64::from(*role_id.expect("Failed to get role"))).await.expect("Failed to remove role");

        msg.reply(&ctx, format!("<@!{}> has been unmuted", user_id)).await?;
    }

    Ok(())
}

#[command]
pub async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.delete_message(&ctx.http.as_ref(), msg.id).await.expect("Failed to delete message");

    if args.is_empty() {
        msg.reply(&ctx, "You need to add a message that I should say.").await?;
    } else {
        msg.channel_id.send_message(&ctx.http, |m| m.content(args.message())).await.expect("Failed to send message");
    }

    Ok(())
}