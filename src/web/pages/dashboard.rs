use axum::extract::Request;
use chrono::{DateTime, Utc};
use maud::{Markup, PreEscaped, html};
use uuid::Uuid;

use crate::{
    database::{self},
    error::CompositeError,
    persons::{Name, Person},
    quotes::{Quote, QuoteLine},
    tags::Tag,
    users::{User, auth::UserAuthenticate},
    web::{
        components::{chip, nav::nav, quote::quote},
        icons,
        pages::base,
    },
};

const LINKS: &[(&str, &str, &str)] = &[
    ("Add Quote", "/quotes/add", icons::QUOTE),
    ("Add Person", "/persons/add", icons::CONTACT),
];

pub async fn page(req: Request) -> Result<Markup, CompositeError> {
    let u = User::authenticate(req.headers()).ok().flatten();
    let conn = database::conn()?;

    let newest_quote = Quote::get_newest(&conn)?;

    Ok(base(
        "Dashboard | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            div class="mx-auto max-w-4xl mt-4 grid grid-cols-1 sm:grid-cols-2 gap-4" {
                div class="flex flex-col" {
                    p {"Newest Quote"}
                    @if let Some(q) = newest_quote {
                        p class="text-neutral-500 font-light mb-4" {
                            "This just in! This quote was added "
                            (format_time_ago(q.get_creation_timestamp())) " ago."
                        }
                        div class="flex-1 [&>div]:h-full" {(quote(&q))}
                    } @else {
                        p class="text-neutral-500 font-light mb-4" {"No quotes yet."}
                    }
                }
                div class="flex flex-col" {
                    p {"Quote of the Day"}
                    p class="text-neutral-500 font-light mb-4" {"This quote was voiced a year ago today."}
                    div class="flex-1 [&>div]:h-full" {(quote(&sample_quote_2()))}
                }
            }
            div class="mx-auto max-w-4xl mt-4" {
                p class="mb-2" {"Quick access"}
                div class="flex gap-4" {
                    @for (title, url, icon) in LINKS {
                        a href=(url)
                            class="border border-neutral-200/25 hover:border-neutral-200/35 bg-neutral-200/5 hover:bg-neutral-200/10 p-4 rounded flex-1 relative overflow-clip" {
                            div class="absolute top-1 right-1 scale-[2] -rotate-12 text-neutral-700" {(PreEscaped(icon))}
                            div class="absolute bottom-1 left-1 scale-[2] -rotate-12 text-neutral-700" {(PreEscaped(icon))}
                            p class="sm:text-2xl font-semibold text-center" {(title)}
                        }
                    }
                }

            }
            div class="mx-auto max-w-4xl mt-4 flex flex-row gap-2" {
                (chip(html!({
                    @match Quote::total_count(&conn) {
                        Ok(count) => {(count) " QUOTES TOTAL"},
                        Err(_) => span class="text-red-400" {"QUOTE COUNT ERR"},
                    }
                })))
                (chip(html!({
                    @match Person::total_count(&conn) {
                        Ok(count) => {(count) " PERSONS TOTAL"},
                        Err(_) => span class="text-red-400" {"PERSON COUNT ERR"},
                    }
                })))
                (chip(html!({
                    @match Tag::total_count(&conn) {
                        Ok(count) => {(count) " TAGS TOTAL"},
                        Err(_) => span class="text-red-400" {"TAG COUNT ERR"}
                    }
                })))
                (chip(html!({
                    @match User::total_count(&conn) {
                        Ok(count) => {(count) " USERS TOTAL"},
                        Err(_) => span class="text-red-400" {"USER COUNT ERR"}
                    }
                })))
            }

            div class="text-4xl xs:text-6xl sm:text-8xl text-neutral-800/25 mt-16 text-center font-semibold font-lora select-none" {"Mnemosyne"}
        ),
    ))
}

fn sample_quote_2() -> Quote {
    Quote {
        id: Uuid::now_v7(),
        public: true,
        location: Some(String::from("Discord VC")),
        context: Some(String::from("O narysowanej dziewczynie")),
        created_by: Uuid::max(),
        timestamp: DateTime::from(Utc::now()),
        lines: vec![
            QuoteLine {
                id: Uuid::now_v7(),
                content: String::from("Czy tu proporcje są zachowane?"),
                attribution: Name {
                    id: Uuid::now_v7(),
                    created_by: Uuid::max(),
                    person_id: Uuid::now_v7(),
                    is_primary: true,
                    name: String::from("Adam"),
                },
            },
            QuoteLine {
                id: Uuid::now_v7(),
                content: String::from("Adam, ona nie ma kolan."),
                attribution: Name {
                    id: Uuid::nil(),
                    created_by: Uuid::max(),
                    person_id: Uuid::now_v7(),
                    is_primary: true,
                    name: String::from("Mollin"),
                },
            },
        ],
    }
}

fn format_time_ago(dt: DateTime<Utc>) -> String {
    let secs = Utc::now().signed_duration_since(dt).num_seconds();
    match secs {
        ..60 => format!("{}s", secs),
        60..3600 => format!("{}m", secs / 60),
        3600..86400 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86400),
    }
}
