use std::env::{args, var};

use icey_pudding::conductors::Conductor;
use icey_pudding::entities::{Content, User};
use icey_pudding::handlers::Handler;
use icey_pudding::repositories::InMemoryRepository;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::client::ClientBuilder;

async fn async_main() {
    let mut args = args();
    args.next();

    let token = match args.next() {
        Some(t) => t,
        None => match var("DISCORD_BOT_TOKEN") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error on getting `DISCORD_BOT_TOKEN`: {}", e);
                eprintln!("fallback to built-in token...");

                match option_env!("BUILD_WITH_DISCORD_BOT_TOKEN") {
                    Some(t) => t.to_string(),
                    None => return eprintln!("cannot get token!"),
                }
            },
        },
    };

    let application_id = match match args.next() {
        Some(i) => i.parse(),
        None => match var("DISCORD_BOT_APPLICATION_ID") {
            Ok(i) => i.parse(),
            Err(e) => {
                eprintln!("error on getting `DISCORD_BOT_APPLICATION_ID`: {}", e);
                eprintln!("fallback to built-in token...");

                match option_env!("BUILD_WITH_DISCORD_BOT_APPLICATION_ID") {
                    Some(i) => i.parse(),
                    None => return eprintln!("cannot get application_id!"),
                }
            },
        },
    } {
        Ok(o) => o,
        Err(e) => return eprintln!("cannot parse application_id: {}", e),
    };

    let eh = Conductor {
        handler: Handler {
            user_repository: Box::new(InMemoryRepository::<User>::new().await),
            content_repository: Box::new(InMemoryRepository::<Content>::new().await),
        },
    };

    let mut c = match ClientBuilder::new(token)
        .application_id(application_id)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES)
        .event_handler(eh)
        .await
    {
        Ok(c) => c,
        Err(e) => return eprintln!("cannot build serenity `Client`: {}", e),
    };

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
