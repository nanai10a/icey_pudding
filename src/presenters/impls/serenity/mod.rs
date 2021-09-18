const EMPTY_FIELD: (&str, &str, bool) = ("\u{200b}", "\u{200b}", true);

pub type View = dyn FnOnce(&mut ::serenity::builder::CreateEmbed) -> &mut ::serenity::builder::CreateEmbed
    + Sync
    + Send;

pub mod content;
pub mod user;
