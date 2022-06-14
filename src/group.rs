use serenity::framework::standard::macros::group;
pub use crate::commands::*;

#[group]
#[commands(avatar, ping, help, say)]
pub struct General;

#[group]
#[commands(ban, kick, mute, unmute)]
pub struct Mod;