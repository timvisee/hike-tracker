use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};

pub struct StaticCache;

/// Maximum age of cache files in seconds
const CACHE_MAX_AGE_SEC: u64 = 7200; // 2 hours in seconds

#[rocket::async_trait]
impl Fairing for StaticCache {
    fn info(&self) -> Info {
        Info {
            name: "Static File Cache Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        // Skip cache control in debug mode
        if cfg!(debug_assertions) {
            return;
        }

        if req.uri().path().starts_with("/static/") {
            res.set_raw_header(
                "Cache-Control",
                format!("public, max-age={CACHE_MAX_AGE_SEC}"),
            );
        }
    }
}
