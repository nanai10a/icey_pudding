use std::env::{args, var};

use icey_pudding::in_memory;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::client::ClientBuilder;

async fn async_main() {
    let AppValues { token } = match get_values() {
        Ok(o) => o,
        Err(e) => return e,
    };

    let mut c = match ClientBuilder::new(token)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES)
        .event_handler(in_memory())
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

struct AppValues {
    token: String,
}

fn get_values() -> Result<AppValues, ()> {
    let mut args = args();
    args.next(); // 最初の引数はcli上のcommand_name

    let token =
        crate::try_get_value!(args; "DISCORD_BOT_TOKEN", "BUILD_WITH_DISCORD_BOT_TOKEN", "token")?;

    Ok(AppValues { token })
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
