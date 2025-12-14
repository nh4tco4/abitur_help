use actix_web::{App, HttpServer, web};

mod db;
mod faq;
mod loader;
mod models;

use actix_cors::Cors;
use db::create_pool;
use faq::{search_faq, list_all_faq, add_faq_entry, delete_faq_entry, toggle_faq_active};

async fn init_db(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(include_str!("../schema.sql"))
        .execute(pool)
        .await?;

    let rec_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM faq_entries")
        .fetch_one(pool)
        .await?;

    if rec_count.0 == 0 {
        println!("Database empty -> loading FAQ data...");
        loader::load_faq_from_json(pool).await?;
    } else {
        println!("Found {} existing FAQ entries skipping seed", rec_count.0);
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let pool = create_pool().await.expect("Failed to create pool");
    init_db(&pool).await.expect("Failed to initialize database");
    println!("FAQ Server running at http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pool.clone()))
            .route("/api/search", web::post().to(search_faq))
            .route("/api/admin/faq", web::get().to(list_all_faq))
            .route("/api/admin/faq", web::post().to(add_faq_entry))
            .route("/api/admin/faq/{id}", web::delete().to(delete_faq_entry))
            .route("/api/admin/faq/{id}/toggle-active", web::patch().to(toggle_faq_active))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
