use serenity::framework::standard::{CommandResult, DispatchError};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::framework::standard::macros::hook;
use crate::CommandCounter;

#[hook]
pub async fn before(ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
}

#[hook]
pub async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => {},
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
pub async fn unknown_command(_ctx: &Context, _msg: &Message, _unknown_command_name: &str) {
    println!("Unknown command");
}

#[hook]
pub async fn normal_message(_ctx: &Context, _msg: &Message) {

}

#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}