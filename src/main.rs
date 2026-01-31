#[macro_use]
extern crate rocket;

mod auth;
mod cache;
mod db;
mod models;
mod routes;
mod schema;
mod stats;

use db::DbConn;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[get("/")]
fn index() -> rocket::response::Redirect {
    rocket::response::Redirect::to("/dashboard")
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();
    rocket::build()
        .attach(DbConn::fairing())
        .attach(rocket::fairing::AdHoc::on_ignite(
            "Run Migrations",
            |rocket| async {
                use diesel::Connection;
                let db_url = rocket
                    .figment()
                    .extract_inner::<String>("databases.sqlite_db.url")
                    .expect("Database URL not configured");
                let mut conn = diesel::sqlite::SqliteConnection::establish(&db_url)
                    .expect("Failed to connect to database");
                db::run_migrations(&mut conn);
                rocket
            },
        ))
        .attach(Template::fairing())
        .attach(cache::StaticCache)
        .mount("/", routes![index])
        .mount("/", routes::auth::routes())
        .mount("/admin/posts", routes::admin::posts::routes())
        .mount("/admin/groups", routes::admin::groups::routes())
        .mount("/scan", routes::scan::routes())
        .mount("/scan", routes::scan::edit::routes())
        .mount("/dashboard", routes::dashboard::routes())
        .mount("/post", routes::post::routes())
        .mount("/ranking", routes::ranking::routes())
        .mount("/static", FileServer::from("static"))
}
