use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct UserFormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<UserFormData>) -> HttpResponse {
    println!("form.name: {}", form.name);
    println!("form.email: {}", form.email);
    HttpResponse::Ok().finish()
}
