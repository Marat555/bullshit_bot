use dotenvy::dotenv;
use rand::Rng;
use sqlx::{Error as SqlxError, SqlitePool};
use std::sync::Arc;
use teloxide::{prelude::*, types::Message};

pub type AppErr = Box<dyn std::error::Error + Send + Sync>;
pub type HandlerResult = Result<(), AppErr>;

static REPLY_CHANCE: f64 = 0.25;

async fn init_db(pool: &SqlitePool) -> Result<(), SqlxError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS responces (
            id INTEGER PRIMARY KEY,
            text TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_random_response(pool: &SqlitePool) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT text FROM responses ORDER BY RANDOM() LIMIT 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

#[tokio::main]
async fn main() {
    if let Err(e) = dotenv() {
        eprintln!("Failed to load .env file: {}", e);
    }

    let token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN is missing or invalid. Please set it in the .env file or environment variables.");

    println!("TELOXIDE_TOKEN loaded successfully.");

    let database_url = "sqlite:responses.db";
    let pool = Arc::new(
        SqlitePool::connect(database_url)
            .await
            .expect("Failed to connect database"),
    );

    init_db(&pool).await.expect("Failed to initialize database");

    let bot = Bot::new(token);

    let handler = Update::filter_message().endpoint(start_handler);

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![pool.clone()])
        .build()
        .dispatch()
        .await;
}

async fn start_handler(msg: Message, bot: Bot, pool: Arc<SqlitePool>) -> HandlerResult {
    let chat_id = msg.chat.id;

    if msg.chat.is_group() || msg.chat.is_supergroup() {
        let random_number: f64 = rand::thread_rng().r#gen();

        if random_number < REPLY_CHANCE {
            if let Some(response) = get_random_response(&pool).await {
                bot.send_message(chat_id, response).await?;
            }
        }
    }

    Ok(())
}
