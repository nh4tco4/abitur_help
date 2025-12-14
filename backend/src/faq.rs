use actix_web::{HttpResponse, Result, web, HttpRequest};
use serde_json::json;
use sqlx::MySqlPool;

use crate::models::{FaqEntry, SearchQuery, NewFaqEntry};

pub async fn search_faq(
    pool: web::Data<MySqlPool>,
    query: web::Json<SearchQuery>,
) -> Result<HttpResponse> {
    let q = query.query.trim();
    if q.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Query cannot be empty"
        })));
    }

    let results: Vec<FaqEntry> = sqlx::query_as(
        r#"
        SELECT id, question, answer, keywords, created_at, is_active
        FROM faq_entries
        WHERE is_active = TRUE
          AND MATCH(question, keywords) AGAINST(? IN NATURAL LANGUAGE MODE)
        ORDER BY 
          MATCH(question, keywords) AGAINST(? IN NATURAL LANGUAGE MODE) DESC,
          id ASC
        LIMIT 10
        "#,
    )
    .bind(q)
    .bind(q)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if results.is_empty() {
        let suggestions: Vec<String> = sqlx::query_scalar(
            "SELECT question FROM faq_entries WHERE is_active = TRUE ORDER BY RAND() LIMIT 3",
        )
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        return Ok(HttpResponse::Ok().json(json!({
            "status": "not_found",
            "message": "No matching FAQ found",
            "suggestions": suggestions
        })));
    }

    let best = &results[0];
    let similar: Vec<String> = results.iter().skip(1).map(|f| f.question.clone()).collect();

    Ok(HttpResponse::Ok().json(json!({
        "status": "found",
        "question": best.question,
        "answer": best.answer,
        "similar_questions": similar
    })))
}

fn check_admin_auth(req: &HttpRequest) -> bool {
    // if let Some(token) = req.headers().get(actix_web::http::header::HeaderName::from_static("x-admin-token")) {
    //     if let Ok(token_str) = token.to_str() {
    //         return token_str == &std::env::var("ADMIN_TOKEN").unwrap_or_default();
    //     }
    // }
    true
}

pub async fn list_all_faq(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if !check_admin_auth(&req) {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Access denied"})));
    }

    let entries: Vec<FaqEntry> = sqlx::query_as(
        "SELECT id, question, answer, keywords, created_at, is_active FROM faq_entries ORDER BY id DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "data": entries })))
}

pub async fn add_faq_entry(
    pool: web::Data<MySqlPool>,
    entry: web::Json<NewFaqEntry>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if !check_admin_auth(&req) {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Access denied"})));
    }

    let question = entry.question.trim();
    let keywords = entry.keywords.trim();
    let answer = entry.answer.trim();

    if question.is_empty() || answer.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Question and answer must not be empty"
        })));
    }

    let res = sqlx::query(
        "INSERT INTO faq_entries (question, answer, keywords) VALUES (?, ?, ?)",
    )
    .bind(question)
    .bind(answer)
    .bind(keywords)
    .execute(&**pool)
    .await;

    match res {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "message": "FAQ entry added successfully"
        }))),
        Err(e) => {
            eprintln!("DB insert error: {e}");
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to add entry"
            })))
        }
    }
}

pub async fn delete_faq_entry(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if !check_admin_auth(&req) {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Access denied"})));
    }

    let id = *id;
    if id <= 0 {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Invalid ID"})));
    }

    let res = sqlx::query("DELETE FROM faq_entries WHERE id = ?")
        .bind(id)
        .execute(&**pool)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if res.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(json!({"message": "Entry deleted"})))
    } else {
        Ok(HttpResponse::NotFound().json(json!({"error": "Entry not found"})))
    }
}

pub async fn toggle_faq_active(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if !check_admin_auth(&req) {
        return Ok(HttpResponse::Forbidden().json(json!({"error": "Access denied"})));
    }

    let id = *id;
    if id <= 0 {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Invalid ID"})));
    }

    let current: Option<(bool,)> = sqlx::query_as("SELECT is_active FROM faq_entries WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if current.is_none() {
        return Ok(HttpResponse::NotFound().json(json!({"error": "Entry not found"})));
    }

    let new_active = !current.unwrap().0;

    sqlx::query("UPDATE faq_entries SET is_active = ? WHERE id = ?")
        .bind(new_active)
        .bind(id)
        .execute(&**pool)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({
        "id": id,
        "is_active": new_active
    })))
}
