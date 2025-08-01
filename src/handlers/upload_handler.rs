use actix_web::{post, web, HttpResponse, Error};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::io::Write;
use uuid::Uuid;
use serde_json::json;

#[post("/upload/profile-photo")]
pub async fn upload_profile_photo(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut filename = None;
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let _content_type = field.content_disposition().unwrap();
        let generated_filename = format!("{}.jpg", Uuid::new_v4());
        let filepath = format!("./uploads/{}", generated_filename);
        filename = Some(generated_filename.clone());

        // Add 'move' to closure to transfer ownership of filepath
        let mut file = web::block(move || std::fs::File::create(&filepath)).await??;
        
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file = web::block(move || {
                file.write_all(&data)?;
                Ok::<_, std::io::Error>(file)
            }).await??;
        }
    }
    let url = filename.map(|f| format!("/uploads/{}", f)).unwrap_or_default();
    Ok(HttpResponse::Ok().json(json!({"url": url})))
}