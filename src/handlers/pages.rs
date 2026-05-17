use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate;

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate;

pub async fn index() -> Html<String> {
    let tmpl = IndexTemplate;
    Html(tmpl.render().unwrap())
}

pub async fn login_page() -> Html<String> {
    let tmpl = LoginTemplate;
    Html(tmpl.render().unwrap())
}

pub async fn register_page() -> Html<String> {
    let tmpl = RegisterTemplate;
    Html(tmpl.render().unwrap())
}

pub async fn dashboard_page() -> Html<String> {
    let tmpl = DashboardTemplate;
    Html(tmpl.render().unwrap())
}

pub async fn stats_page() -> Html<String> {
    let tmpl = StatsTemplate;
    Html(tmpl.render().unwrap())
}
