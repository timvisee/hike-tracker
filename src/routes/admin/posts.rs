use rocket::form::Form;
use rocket::response::Redirect;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::models::Post;
use crate::AppState;

#[derive(FromForm)]
pub struct NewPost {
    name: String,
    order: i32,
}

#[get("/")]
pub fn posts(state: &State<AppState>) -> Template {
    let db = state.db.lock().unwrap();
    let posts = Post::get_all(db.conn()).unwrap_or_default();
    Template::render("admin/posts", context! { posts: posts })
}

#[post("/", data = "<form>")]
pub fn create_post(state: &State<AppState>, form: Form<NewPost>) -> Redirect {
    let db = state.db.lock().unwrap();

    let post = Post::new(form.name.clone(), form.order);
    post.insert(db.conn()).unwrap();

    Redirect::to("/admin/posts")
}

#[get("/<id>/delete")]
pub fn delete_post(state: &State<AppState>, id: &str) -> Redirect {
    let db = state.db.lock().unwrap();
    Post::delete(db.conn(), id).unwrap();
    Redirect::to("/admin/posts")
}

pub fn routes() -> Vec<Route> {
    routes![posts, create_post, delete_post]
}
