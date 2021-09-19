use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::{json, Number, Value};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use tracing::Instrument;

use crate::controllers::serenity::SerenityReturnController;
use crate::utils::{AlsoChain, LetChain};

pub struct Conductor {
    pub contr: SerenityReturnController,
}

#[async_trait]
impl EventHandler for Conductor {
    async fn message(&self, ctx: Context, msg: Message) {
        tracing::trace!("msg - {:?}", msg);

        if msg.author.bot {
            return;
        }

        let res = match match self.contr.parse(&msg, &ctx).await {
            Some(r) => r,
            None => return,
        } {
            Ok(mut sv) =>
                msg.channel_id
                    .send_message(&ctx, |cm| {
                        #[allow(clippy::unit_arg)]
                        sv.drain(..)
                            .for_each(|v| cm.add_embed(v).let_(::core::mem::drop))
                            .let_(|()| cm)
                            .also_(|cm| {
                                append_message_reference(
                                    &mut cm.0,
                                    msg.id,
                                    msg.channel_id,
                                    msg.guild_id,
                                )
                            })
                    })
                    .instrument(tracing::trace_span!("send_message"))
                    .await,
            Err(e) =>
                msg.channel_id
                    .send_message(&ctx, |cm| {
                        cm.content(format!("```{}```", e)).also_(|cm| {
                            append_message_reference(
                                &mut cm.0,
                                msg.id,
                                msg.channel_id,
                                msg.guild_id,
                            )
                        })
                    })
                    .instrument(tracing::trace_span!("send_message"))
                    .await,
        };

        let e = match res {
            Ok(o) =>
                return tracing::info!(
                    "replied - id {} | channel_id {} | guild_id {} | time {}",
                    o.id,
                    o.channel_id,
                    o.guild_id
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "None".to_string()),
                    o.timestamp,
                ),
            Err(e) => e,
        };

        tracing::warn!("repling err - {:?}", e);

        let res = msg
            .channel_id
            .send_message(ctx, |cm| {
                cm.content(format!(
                    "error occurred.
please send this message to administrator.
```
# from_msg
  - mid  : {}
  - cid  : {}
  - gid  : {}
  - time : {}

# current
  - time : {}

# err_msg
{}
```",
                    msg.id,
                    msg.channel_id,
                    msg.guild_id
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "None".to_string()),
                    msg.timestamp,
                    ::chrono::Utc::now(),
                    e
                ))
                .also_(|cm| {
                    append_message_reference(&mut cm.0, msg.id, msg.channel_id, msg.guild_id)
                })
            })
            .instrument(tracing::trace_span!("send_message"))
            .await;

        match res {
            Ok(o) => tracing::warn!(
                "reported err - id {} | channel_id {} | guild_id {} | time {}",
                o.id,
                o.channel_id,
                o.guild_id
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "None".to_string()),
                o.timestamp
            ),
            Err(e) => tracing::error!("cannot report err - {}", e),
        }
    }
}

fn append_message_reference(
    raw: &mut HashMap<&str, Value>,
    id: MessageId,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
) {
    let mr = json!({
        "message_id": id,
        "channel_id": channel_id,
        "guild_id": match guild_id {
            Some(GuildId(i)) => Value::Number(Number::from(i)),
            None => Value::Null
        },
    });

    raw.insert("message_reference", mr);
}
