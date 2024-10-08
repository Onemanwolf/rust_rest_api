#[macro_use]
extern crate lazy_static;

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use std::hash::{Hash, Hasher};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;



#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, utoipa::ToSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
}

lazy_static! {
    static ref USERS: Mutex<HashSet<User>> = Mutex::new({
        let mut set = HashSet::new();
        set.insert(User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
        });
        set
    });
}

#[utoipa::path(
    get,
    path = "/user",
    responses(
        (status = 200, description = "Get all users", body = [User])
    )
)]
async fn get_user() -> impl Responder {
    let users = USERS.lock().unwrap();
    let users_vec: Vec<&User> = users.iter().collect();
    HttpResponse::Ok().json(users_vec)
}

#[utoipa::path(
    post,
    path = "/user",
    request_body = User,
    responses(
        (status = 201, description = "Create a new user")
    )
)]
async fn create_user(user: web::Json<User>) -> impl Responder {
    let mut users = USERS.lock().unwrap();
    let new_user = user.into_inner();

    if users.iter().any(|u| u.id == new_user.id) {
        HttpResponse::Conflict().body("User with this ID already exists")
    } else {
        users.insert(new_user);
        HttpResponse::Created().finish()
    }
}

#[utoipa::path(
    put,
    path = "/user",
    request_body = User,
    responses(
        (status = 200, description = "Updated a new user")
    )
)]

async fn update_user(user: web::Json<User>) -> impl Responder {
    let mut users = USERS.lock().unwrap();
    let updated_user = user.into_inner();

    if users.replace(updated_user).is_some() {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().body("User with this ID does not exist")
    }
}

#[derive(OpenApi)]
#[openapi(paths(get_user, create_user, update_user), components(schemas(User)))]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
        .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
            .route("/user", web::get().to(get_user))
            .route("/user", web::post().to(create_user))
            .route("/user", web::put().to(update_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}