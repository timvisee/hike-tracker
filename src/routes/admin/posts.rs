use rocket::form::Form;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::auth::Admin;
use crate::db::DbConn;
use crate::models::{Group, NewPost, Post, Scan};

#[derive(FromForm)]
pub struct NewPostForm {
    name: String,
    order: i32,
}

#[derive(Serialize)]
pub struct PostWithStats {
    pub post: Post,
    pub arrived_count: usize,
    pub total_groups: usize,
    pub has_password: bool,
}

#[get("/")]
pub async fn posts(_admin: Admin, conn: DbConn) -> Template {
    let posts = conn.run(Post::get_all).await.unwrap_or_default();
    let groups = conn.run(Group::get_all).await.unwrap_or_default();
    let total_groups = groups.len();

    let mut posts_with_stats = Vec::new();
    for post in posts {
        let post_id = post.id.clone();
        let scans = conn
            .run(move |c| Scan::get_by_post(c, &post_id))
            .await
            .unwrap_or_default();
        let arrived_count = scans.len();
        let has_password = post.password_hash.is_some();

        posts_with_stats.push(PostWithStats {
            post,
            arrived_count,
            total_groups,
            has_password,
        });
    }

    Template::render(
        "admin/posts",
        context! { posts: posts_with_stats, is_admin: true },
    )
}

#[post("/", data = "<form>")]
pub async fn create_post(_admin: Admin, conn: DbConn, form: Form<NewPostForm>) -> Redirect {
    let name = form.name.clone();
    let order = form.order;
    let result = conn
        .run(move |c| {
            let post = NewPost::new(name, order);
            Post::insert(c, post)
        })
        .await;
    if let Err(err) = result {
        eprintln!("Failed to create post: {err}");
    }

    Redirect::to("/admin/posts")
}

#[get("/<id>/delete")]
pub async fn delete_post(_admin: Admin, conn: DbConn, id: String) -> Redirect {
    conn.run(move |c| Post::delete(c, &id)).await.ok();
    Redirect::to("/admin/posts")
}

#[derive(FromForm)]
pub struct SetPasswordForm {
    password: String,
}

#[post("/<id>/password", data = "<form>")]
pub async fn set_password(
    _admin: Admin,
    conn: DbConn,
    id: String,
    form: Form<SetPasswordForm>,
) -> Redirect {
    let password = form.password.clone();
    if password.is_empty() {
        // Clear password if empty
        conn.run(move |c| Post::clear_password(c, &id)).await.ok();
    } else {
        conn.run(move |c| Post::set_password(c, &id, &password))
            .await
            .ok();
    }
    Redirect::to("/admin/posts")
}

#[get("/<id>/password/clear")]
pub async fn clear_password(_admin: Admin, conn: DbConn, id: String) -> Redirect {
    conn.run(move |c| Post::clear_password(c, &id)).await.ok();
    Redirect::to("/admin/posts")
}

pub fn routes() -> Vec<Route> {
    routes![
        posts,
        create_post,
        delete_post,
        set_password,
        clear_password
    ]
}
