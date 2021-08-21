use std::env::{args, var};

use icey_pudding::conductors::{application_command_create, Conductor};
use icey_pudding::entities::{Content, User};
use icey_pudding::handlers::Handler;
use icey_pudding::repositories::InMemoryRepository;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::client::ClientBuilder;
use serenity::model::id::GuildId;

async fn async_main() {
    let AppValues {
        token,
        app_id,
        app_post_to,
    } = match get_values() {
        Ok(o) => o,
        Err(e) => return e,
    };

    let eh = Conductor {
        handler: Handler {
            user_repository: Box::new(InMemoryRepository::<User>::new().await),
            content_repository: Box::new(InMemoryRepository::<Content>::new().await),
        },
    };

    let mut c = match ClientBuilder::new(token)
        .application_id(app_id)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES)
        .event_handler(eh)
        .await
    {
        Ok(c) => c,
        Err(e) => return eprintln!("cannot build serenity `Client`: {}", e),
    };

    match application_command_create(&c.cache_and_http.http, None).await {
        Ok(o) => println!("successfully post application_cmd to global: {:?}", o),
        Err(e) => eprintln!("cannot post application_cmd to global: {}", e),
    }

    if let Some(guild_id) = app_post_to {
        match application_command_create(&c.cache_and_http.http, Some(GuildId(guild_id))).await {
            Ok(o) => println!(
                "successfully post application_cmd to guild (id: {}): {:?}",
                guild_id, o
            ),
            Err(e) => eprintln!(
                "cannot post application_cmd to guild (id: {}): {}",
                guild_id, e
            ),
        }
    }

    match c.start_autosharded().await {
        Ok(o) => o,
        Err(e) => eprintln!("serenity `Client` returned: {}", e),
    }
}

fn main() {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            let num = unsafe { NUM };
            unsafe { NUM += 1 }
            format!("icey_pudding-worker-{}", num)
        })
        .build()
    {
        Ok(r) => r,
        Err(e) => return eprintln!("{}", e),
    };

    rt.block_on(async_main())
}

static mut NUM: u32 = 0;

struct AppValues {
    token: String,
    app_id: u64,
    app_post_to: Option<u64>,
}

fn get_values() -> Result<AppValues, ()> {
    let mut args = args();
    args.next(); // 最初の引数はcli上のcommand_name

    let token =
        crate::try_get_value!(args; "DISCORD_BOT_TOKEN", "BUILD_WITH_DISCORD_BOT_TOKEN", "token")?;

    let app_id = match
        crate::try_get_value!(args; "DISCORD_BOT_APPLICATION_ID", "BUILD_WITH_DISCORD_BOT_APPLICATION_ID", "application_id")?.parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("cannot parse `application_id`: {}", e);
            return Err(());
        },
    };

    let app_post_to = match crate::try_get_value!(args; "DISCORD_CMD_POST", "BUILD_WITH_DISCORD_CMD_POST", "application_command_post_to")
    {
        Ok(o) => match o.parse() {
            Ok(o) => Some(o),
            Err(e) => {
                eprintln!("cannot parse `application_command_post_to`: {}", e);
                return Err(());
            },
        },
        Err(_) => None,
    };

    Ok(AppValues {
        token,
        app_id,
        app_post_to,
    })
}

#[macro_export]
macro_rules! try_get_value {
    ($a:expr; $n:literal, $bn:literal, $pn:literal) => {{
        match $a.next() {
            Some(t) => Ok(t),
            None => match var($n) {
                Ok(t) => Ok(t),
                Err(e) => {
                    eprintln!("error on getting `{}`: {}", $n, e);
                    eprintln!("fallback to built-in `{}`...", stringify!($pn));

                    match option_env!($bn) {
                        Some(t) => Ok(t.to_string()),
                        None => {
                            eprintln!("cannot get `{}`!", stringify!($pn));
                            Err(())
                        },
                    }
                },
            },
        }
    }};
}
