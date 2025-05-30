/*

POST /url
{
    "url": "https://example.com",
}
*/
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder, get, post, web,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres, postgres::PgPoolOptions};

#[derive(Deserialize, Serialize, FromRow)]
struct UrlData {
    url: String,
    image_base64: String
}

#[post("/")]
async fn hello(pool: Data<Pool<Postgres>>, req: Json<UrlData>) -> impl Responder {
    match sqlx::query("INSERT INTO urls (url,image) VALUES ($1, $2)")
        .bind(&req.url)
        .bind(&req.image_base64)
        .execute(pool.get_ref())
        .await{
            Ok(res) => res,
            Err(error) => return HttpResponse::from_error(actix_web::error::ErrorInternalServerError(error))
        };

    println!("Received request: {}", req.url);
    HttpResponse::Created().body("Hello world!")
}

#[get("/")]
async fn get_urls(pool: Data<Pool<Postgres>>) -> Result<HttpResponse, Error> {
    let urls = sqlx::query_as::<_, UrlData>("SELECT * FROM urls")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(urls))
}

#[get("/images/{name}")]
async fn get_image(name: web::Path<String>) -> impl Responder {
    let image = match std::fs::read(format!("images/{}.png", name)) {
        Ok(image) => image,
        Err(error) => return HttpResponse::from_error(actix_web::error::ErrorNotFound(error)),
    };

    HttpResponse::Ok().content_type("image/png").body(image)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgresql://admin:taschino@localhost:5432/taschino")
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(hello)
            .service(get_urls)
            .service(get_image)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
