use serde::Deserialize;
use sqlx::{MySqlPool, query};
use std::error::Error;

static FAQ_JSON: &str = include_str!("../faq.json");

#[derive(Deserialize)]
struct FaqJsonItem {
    question: String,
    answer: String,
    keywords: String,
}

pub async fn load_faq_from_json(pool: &MySqlPool) -> Result<(), Box<dyn Error>> {
    let items: Vec<FaqJsonItem> = serde_json::from_str(FAQ_JSON)?;

    for item in items.iter() {
        query("INSERT INTO faq_entries (question, answer, keywords) VALUES (?, ?, ?)")
            .bind(&item.question)
            .bind(&item.answer)
            .bind(&item.keywords)
            .execute(pool)
            .await?;
    }

    println!("Loaded {} FAQ entries from JSON", items.len());
    Ok(())
}
