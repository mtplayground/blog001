pub const HOME: &str = "/";
pub const POST_TEMPLATE: &str = "/post/:slug";
pub const TAG_TEMPLATE: &str = "/tag/:tag";
pub const LOGIN: &str = "/login";

pub const ADMIN_DASHBOARD: &str = "/admin";
pub const ADMIN_POSTS: &str = "/admin/posts";
pub const ADMIN_EDITOR: &str = "/admin/editor";

pub const POST_AXUM: &str = "/post/{slug}";
pub const TAG_AXUM: &str = "/tag/{tag}";

pub fn public_routes() -> [&'static str; 3] {
    [HOME, POST_TEMPLATE, TAG_TEMPLATE]
}

pub fn admin_routes() -> [&'static str; 4] {
    [ADMIN_DASHBOARD, ADMIN_POSTS, ADMIN_EDITOR, LOGIN]
}
