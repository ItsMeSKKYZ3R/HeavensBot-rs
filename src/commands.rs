use std::ops::Index;
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
    let face = match args.single::<UserId>() {
        Ok(user_id) => user_id.to_user(ctx).await?.face(),
        Err(_) => msg.author.face(),
    };

    msg.channel_id.say(&ctx.http, face).await?;

    Ok(())
}

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let before = Instant::now();
    let mut m = msg.channel_id.say(&ctx.http, "pong!").await?;
    let after = Instant::now();

    let content = m.content.clone();
    m.edit(ctx, |m| m.content(format!("{} - {}ms", content, (after - before).as_millis()))).await?;

    Ok(())
}

#[command]
pub async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
                                ("!ping", "Show the bot's ping", false)
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
                        e.title("All HeavensBot's moderation commands")
                            .description("This is a description")
                            .fields(vec![
                                ("!help [mod | general]", "Show this message", false),
                                ("!ping", "Show the bot's ping", false)
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
pub async fn mute(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    if _args.is_empty() {
        _msg.reply(&_ctx, "You need to mention someone to mute!").await?;
    } else {
        // Add role to the mentioned member
        let user_id = u64::from(_args.single::<UserId>().unwrap());
        let mut reason = _args.rest();

        if reason.is_empty() {
            reason = "No reason";
        }

        // get the muted role id
        let role_id = u64::from(_ctx.http.get_guild(u64::from(_msg.guild_id.unwrap())).await.expect("Cannot get guild").roles.);

        println!("{}", role_id);

        match _msg.guild_id {
            Some(guild_id) => {
                let guild_id = u64::from(guild_id);
                _ctx.http.add_member_role(guild_id, user_id, role_id).await.unwrap();
            },
            _ => {
                _msg.reply(&_ctx.http, "Cannot get guild id").await.unwrap();

                ()
            },
        }
    }
    Ok(())
}

#[command]
pub async fn unmute(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    Ok(())
}