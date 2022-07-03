use tracing_subscriber::EnvFilter;

async fn async_main() {
    let AppValues { token, flag } = get_values();

    use serenity::model::gateway::GatewayIntents;
    let cb = ::serenity::client::ClientBuilder::new(
        token,
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES,
    );

    let cb = match flag {
        Flag::InMemory => cb.event_handler(::icey_pudding::in_memory()),
        Flag::Mongo { uri, name } =>
            cb.event_handler(::icey_pudding::mongo(uri, name).await.expect("eh error")),
    };

    let mut c = cb.await.expect("cannot build serenity client.");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .init();

    c.start_autosharded()
        .await
        .expect("serenity client returned.");
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
    flag: Flag,
}

enum Flag {
    InMemory,
    Mongo { uri: String, name: String },
}

fn get_values() -> AppValues {
    use std::env::var;

    let token = var("DISCORD_BOT_TOKEN").expect("error on: DISCORD_BOT_TOKEN");

    let flag = match var("FLAG").expect("error on: FLAG").as_str() {
        "InMemory" => Flag::InMemory,
        "Mongo" => {
            let uri = var("MONGO_URI").expect("error on: MONGO_URI");
            let name = var("MONGO_DB_NAME").expect("error on: MONGO_DB_NAME");

            Flag::Mongo { uri, name }
        },
        v => panic!("unexpected value: {}", v),
    };

    AppValues { token, flag }
}
