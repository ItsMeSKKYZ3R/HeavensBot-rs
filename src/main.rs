mod commands;
mod hook;
mod group;

// use commands::*;
use hook::*;
use group::*;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{GatewayIntents, ShardManager},
    framework::standard::{
        buckets::{LimitedFor},
        StandardFramework,
    },
    http::Http,
    model::{
        gateway::Ready,
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
        .group(&MOD_GROUP)
        .group(&FUN_GROUP);

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