type View = dyn FnOnce(&mut ::serenity::builder::CreateEmbed) -> &mut ::serenity::builder::CreateEmbed
    + Sync
    + Send;

pub mod content;
pub mod user;
