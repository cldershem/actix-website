use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::{r2d2, r2d2::ConnectionManager, SqliteConnection};
use futures::future::Future;
#[macro_use]
extern crate diesel;
use diesel::prelude::*;

mod models;
mod schema;

use models::{NewUser, User};

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

fn list(pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        use schema::users::dsl::*;

        let conn: &SqliteConnection = &pool.get().unwrap();

        users.load(conn)
    })
    .map(|users: Vec<User>| HttpResponse::Ok().json(users))
    .from_err()
}

fn create(
    pool: web::Data<Pool>,
    new_user: web::Json<NewUser>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        use schema::users;

        let conn: &SqliteConnection = &pool.get().unwrap();

        diesel::insert_into(users::table)
            .values(new_user.into_inner())
            .execute(conn)
    })
    .map(|user| HttpResponse::Ok().json(user))
    .from_err()
}

fn detail(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        use schema::users::dsl::*;

        let conn: &SqliteConnection = &pool.get().unwrap();
        let user_id = path.0;

        users.find(user_id).first::<User>(conn)
    })
    .map(|user| HttpResponse::Ok().json(user))
    .from_err()
}

fn update(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
    update_user: web::Json<User>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        use schema::users::dsl::*;

        let conn: &SqliteConnection = &pool.get().unwrap();

        diesel::update(users.filter(id.eq(path.0)))
            .set(&update_user.into_inner())
            .execute(conn)
    })
    .map(|user| HttpResponse::Ok().json(user))
    .from_err()
}

fn delete(
    pool: web::Data<Pool>,
    path: web::Path<(i32,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        use schema::users::dsl::*;

        let conn: &SqliteConnection = &pool.get().unwrap();
        let user_id = path.0;

        diesel::delete(users.filter(id.eq(user_id))).execute(conn)
    })
    .map(|user| HttpResponse::Ok().json(user))
    .from_err()
}

fn index() -> impl Responder {
    "index"
}

fn long_running_task() -> impl Responder {
    use std::{thread, time};
    let delay = time::Duration::from_millis(20000);
    thread::sleep(delay);

    format!("returned after {:?}", delay)
}

fn api_index() -> impl Responder {
    "api index"
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let db_url = "temp.sqlite";
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::new("%a %r %s %b %T"))
            .route("/", web::get().to(index))
            .route("/sleep", web::get().to(long_running_task))
            .service(
                web::scope("api")
                    .service(web::resource("").route(web::get().to(api_index)))
                    .service(
                        web::resource("/users")
                            .route(web::get().to_async(list))
                            .route(web::post().to_async(create)),
                    )
                    .service(
                        web::resource("/users/{user_id}")
                            .route(web::get().to_async(detail))
                            .route(web::put().to_async(update))
                            .route(web::delete().to_async(delete)),
                    ),
            )
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();

    println!("127.0.0.1:8088");
}
