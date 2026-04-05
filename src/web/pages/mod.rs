use axum::{
    Router,
    routing::{get, post},
};
use maud::{DOCTYPE, Markup, html};

pub mod dashboard;
pub mod index;
pub mod login;
pub mod logs;
pub mod persons;
pub mod quotes;
pub mod tags;
pub mod users;
pub mod usersettings;

pub fn pages() -> Router {
    Router::new()
        .route("/", get(index::page))
        .route("/login", get(login::page))
        .route("/dashboard", get(dashboard::page))
        .route("/user-settings", get(usersettings::page))
        .route("/user-settings/handle", post(usersettings::change_handle))
        .route("/user-settings/passwd", post(usersettings::change_password))
        .route("/users", get(users::page))
        .route("/users/{id}", get(users::profile::page))
        .route("/users/create", get(users::create::page))
        .route("/users/create-form", post(users::create::create_user))
        .route("/tags", get(tags::page))
        .route("/tags/create", post(tags::create))
        .route("/persons", get(persons::page))
        .route("/persons/create", post(persons::create))
        .route("/logs", get(logs::page))
        //
        .route("/quotes", get(quotes::page))
        .route("/quotes/add", get(quotes::add::page))
        .route("/quotes/add-form", post(quotes::add::form))
}

pub fn base(title: &str, inner: Markup) -> Markup {
    html!(
        (DOCTYPE)
        head {
            title {(title)}
            meta charset="utf-8";
            link rel="stylesheet" href="/styles.css";
            // link rel="icon" type="image/png" href="/icon.png";
            // link rel="icon" type="image/svg" href="/icon.svg";
            meta name="viewport" content="width=device-width, initial-scale=1.0";

            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Lexend:wght@100..900&family=Lora:ital,wght@0,400..700;1,400..700&display=swap";
        }
        body class="bg-neutral-900 text-neutral-200 font-lexend min-h-screen" {
            (inner)
        }
    )
}
