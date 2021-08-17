use std::env::{args, var};

use icey_pudding::conductors::Conductor;
use icey_pudding::entities::{Content, User};
use icey_pudding::handlers::Handler;
use icey_pudding::repositories::{InMemoryRepository, Repository, UserQuery};
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::client::ClientBuilder;

async fn async_main() {
    let token = match args().next() {
        Some(t) => t,
        None => match var("DISCORD_BOT_TOKEN") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error on getting `DISCORD_BOT_TOKEN`:{}", e);
                eprintln!("fallback to built-in token...");

                match option_env!("BUILD_WITH_DISCORD_BOT_TOKEN") {
                    Some(t) => t.to_string(),
                    None => return eprintln!("cannot get token!"),
                }
            },
        },
    };

    let eh = Conductor {
        handler: Handler {
            user_repository: Box::new(InMemoryRepository::<User>::new().await),
            content_repository: Box::new(InMemoryRepository::<Content>::new().await),
        },
    };

    let mut c = match ClientBuilder::new(token)
        .intents(GatewayIntents::empty())
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
        .thread_name_fn(thread_name_generator)
        .build()
    {
        Ok(r) => r,
        Err(e) => return eprintln!("{}", e),
    };

    rt.block_on(async_main())
}

fn thread_name_generator() -> String {
    static mut NUMS: Vec<u32> = vec![];

    let mut num_tmp = 0u32;
    let mut iter = unsafe {
        NUMS.sort_unstable();
        NUMS.iter()
    };
    let num = loop {
        let n = match iter.next() {
            Some(n) => n,
            None => break num_tmp + 1,
        };

        if *n == num_tmp {
            num_tmp += 1;
            continue;
        } else {
            break num_tmp;
        }
    };

    format!("{}", num)
}
