use serenity::framework::standard::macros::group;
use crate::commands::*;

#[group]
#[commands(avatar, ping, help)]
struct General;

#[group]
#[commands(ban, kick, mute, unmute)]
struct Mod;