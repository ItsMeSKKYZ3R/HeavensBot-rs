use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use std::time::Instant;

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{GatewayIntents, ShardManager},
    framework::standard::{
        buckets::{LimitedFor},
        macros::{command, group, hook},
        Args,
        CommandResult,
        DispatchError,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::Message,
        gateway::Ready,
        id::UserId,
    },
};
use tokio::sync::Mutex;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

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

#[group]
#[commands(avatar, ping, help)]
struct General;

#[group]
#[commands(ban, kick, mute, unmute)]
struct Mod;

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => {},
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, _unknown_command_name: &str) {

}

#[hook]
async fn normal_message(_ctx: &Context, _msg: &Message) {

}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

use serenity::{futures::future::BoxFuture, FutureExt};
use serenity::utils::Colour;

fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                    .await;
            }
        };
    }
        .boxed()
}

#[tokio::main]
async fn main() {
    let token = "token";
    let http = Http::new_with_token(&token);

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
            .delimiters(vec![" "])
            .owners(owners))
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .bucket("emoji", |b| b.delay(5)).await
        .bucket("complicated", |b| b.limit(2).time_span(30).delay(5)
            .limit_for(LimitedFor::Channel)
            .await_ratelimits(1)
            .delay_action(delay_action)).await
        .group(&GENERAL_GROUP)
        .group(&MOD_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

// Commands

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
pub async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
            .await;
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
            .await;
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
            .await;
    } else {
        msg.channel_id.say(&ctx.http, "This category doesn't exists.").await;
    }

    Ok(())
}

#[command]
pub async fn mute(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    Ok(())
}

#[command]
pub async fn unmute(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    Ok(())
}