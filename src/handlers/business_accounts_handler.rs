use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::business_accounts::{CreateBusinessAccount, BusinessAccount, UpdateBusinessAccount};
use serde_json::json;

// Language extraction function (same as your user handler)
fn get_lang(req: &HttpRequest) -> String {
    req.headers()
        .get("Accept-Language")
        .and_then(|val| val.to_str().ok())
        .map(|s| {
            if s.contains("hi") {
                "hi"
            } else {
                "en"
            }
        })
        .unwrap_or("en")
        .to_string()
}

// Create Business Account
pub async fn create_business_account(
    req: HttpRequest,
    db: web::Data<PgPool>,
    json: web::Json<CreateBusinessAccount>,
) -> HttpResponse {
    let lang = get_lang(&req);
    println!("Language preference: {}", lang);

    let account = sqlx::query_as::<_, BusinessAccount>(
        r#"
        INSERT INTO business_accounts 
        (id, company_name, contact_person_name, email, password_hash, address, gst_number, website, phone, logo_url, description, industry, size, is_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&json.company_name)
    .bind(&json.contact_person_name)
    .bind(&json.email)
    .bind(&json.password_hash)
    .bind(&json.address)
    .bind(&json.gst_number)
    .bind(&json.website)
    .bind(&json.phone)
    .bind(&json.logo_url)
    .bind(&json.description)
    .bind(&json.industry)
    .bind(&json.size)
    .bind(json.is_verified.unwrap_or(false))
    .fetch_one(db.get_ref())
    .await;

    match account {
        Ok(acc) => HttpResponse::Ok().json(acc),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Error creating business account")
        }
    }
}

// Get All Business Accounts
pub async fn get_business_accounts(db: web::Data<PgPool>) -> HttpResponse {
    let accounts = sqlx::query_as::<_, BusinessAccount>(
        "SELECT * FROM business_accounts ORDER BY created_at DESC"
    )
    .fetch_all(db.get_ref())
    .await;

    match accounts {
        Ok(accs) => HttpResponse::Ok().json(accs),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Error retrieving business accounts")
        }
    }
}

// Get Business Account by ID
pub async fn get_business_account_by_id(
    db: web::Data<PgPool>,
    account_id: web::Path<Uuid>,
) -> HttpResponse {
    let account = sqlx::query_as::<_, BusinessAccount>(
        "SELECT * FROM business_accounts WHERE id = $1"
    )
    .bind(*account_id)
    .fetch_optional(db.get_ref())
    .await;

    match account {
        Ok(Some(acc)) => HttpResponse::Ok().json(acc),
        Ok(None) => HttpResponse::NotFound().json("Business account not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json("Error retrieving business account")
        }
    }
}

// Delete Business Account by Email
pub async fn delete_business_account_by_email(
    db: web::Data<PgPool>,
    email: web::Path<String>,
) -> HttpResponse {
    let result = sqlx::query!(
        "DELETE FROM business_accounts WHERE email = $1 RETURNING email",
        email.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json("Business account deleted successfully"),
        Ok(None) => HttpResponse::NotFound().json("Business account not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json("Error deleting business account")
        }
    }
}

// Update Business Account by Email
pub async fn update_business_account_by_email(
    db: web::Data<PgPool>,
    email: web::Path<String>,
    json: web::Json<UpdateBusinessAccount>,
) -> HttpResponse {
    let mut update_query = String::from("UPDATE business_accounts SET updated_at = CURRENT_TIMESTAMP");
    let mut params: Vec<(String, String)> = Vec::new();

    if let Some(company_name) = &json.company_name {
        params.push(("company_name".to_string(), company_name.clone()));
    }
    if let Some(contact_person_name) = &json.contact_person_name {
        params.push(("contact_person_name".to_string(), contact_person_name.clone()));
    }
    if let Some(email_new) = &json.email {
        params.push(("email".to_string(), email_new.clone()));
    }
    if let Some(password_hash) = &json.password_hash {
        params.push(("password_hash".to_string(), password_hash.clone()));
    }
    if let Some(address) = &json.address {
        params.push(("address".to_string(), address.clone()));
    }
    if let Some(gst_number) = &json.gst_number {
        params.push(("gst_number".to_string(), gst_number.clone()));
    }
    if let Some(website) = &json.website {
        params.push(("website".to_string(), website.clone()));
    }
    if let Some(phone) = &json.phone {
        params.push(("phone".to_string(), phone.clone()));
    }
    if let Some(logo_url) = &json.logo_url {
        params.push(("logo_url".to_string(), logo_url.clone()));
    }
    if let Some(description) = &json.description {
        params.push(("description".to_string(), description.clone()));
    }
    if let Some(industry) = &json.industry {
        params.push(("industry".to_string(), industry.clone()));
    }
    if let Some(size) = &json.size {
        params.push(("size".to_string(), size.clone()));
    }
    if let Some(is_verified) = &json.is_verified {
        params.push(("is_verified".to_string(), is_verified.to_string()));
    }

    if params.is_empty() {
        return HttpResponse::BadRequest().json("No fields to update");
    }

    for (i, (field, _)) in params.iter().enumerate() {
        update_query.push_str(&format!(", {} = ${}", field, i + 1));
    }
    update_query.push_str(&format!(" WHERE email = ${} RETURNING *", params.len() + 1));

    let mut query = sqlx::query_as::<_, BusinessAccount>(&update_query);

    for (_, value) in &params {
        query = query.bind(value);
    }
    query = query.bind(email.as_ref());

    match query.fetch_one(db.get_ref()).await {
        Ok(updated_account) => HttpResponse::Ok().json(updated_account),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json("Business account not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json("Error updating business account")
        }
    }
}
