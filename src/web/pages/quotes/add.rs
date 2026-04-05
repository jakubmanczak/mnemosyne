use axum::{
    Json,
    extract::Request,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Form;
use chrono::{TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use maud::{PreEscaped, html};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database,
    error::CompositeError,
    logs::{LogAction, LogEntry},
    persons::Name,
    quotes::Quote,
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
    },
    web::{components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, CompositeError> {
    let u = User::authenticate(req.headers())?;
    let conn = database::conn()?;
    let names = Name::get_all(&conn)?;

    Ok(base(
        "Add Quote | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            div class="max-w-4xl mx-auto px-2" {
                div class="my-4 flex justify-between" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::SCROLL_TEXT))}
                        span class="text-2xl font-semibold font-lora" {"Quote Maker"}
                    }
                }
                form method="post" action="/quotes/add-form"
                class="border border-neutral-200/25 bg-neutral-200/5 rounded-md p-4 flex flex-col" {
                    @for i in 1..=2 {
                        div class="flex justify-between gap-4" {
                            div class="flex flex-col flex-1" {
                                label class="w-full" {
                                    p class="mb-1" {(format!("Quote Line #{i}"))}
                                    input type="text" name="quoteline" placeholder="They said..." autocomplete="off"
                                        class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                                }
                            }
                            div class="flex flex-col" {
                                label {
                                    p class="mb-1" {(format!("Quote Author #{i}"))}
                                    select name="quoteauthor" autocomplete="off"
                                        class="px-2 py-1.5 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25"{
                                            option {"--"}
                                            @for name in &names {
                                                option value=(name.id.to_string()) {(name.name)}
                                            }
                                        }
                                }
                            }
                        }
                    }
                    hr class="border-neutral-200/25 my-4";
                    div class="flex gap-4 justify-between" {
                        div class="flex flex-col flex-1" {
                            label class="w-full"{
                                p class="mb-1" {"Location"}
                                input type="text" name="location" autocomplete="off" placeholder="Right there!"
                                    class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                        }
                        div class="flex flex-col flex-1" {
                            label class="w-full" {
                                p class="mb-1" {"Time of utterance"}
                                input type="hidden" name="tz_offset" id="tz_offset" value="0";
                                script { (PreEscaped("document.getElementById('tz_offset').value = new Date().getTimezoneOffset();")) }
                                input type="datetime-local" name="time" autocomplete="off"
                                    class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                        }
                    }
                    div class="flex gap-4 justify-between" {
                        div class="flex flex-col flex-1" {
                            label class="w-full" {
                                p class="mb-1" {"Context"}
                                input type="text" name="context" autocomplete="off" placeholder="It was like this.."
                                    class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                        }
                        button type="submit" class="border mt-auto mb-2 cursor-pointer rounded h-fit px-2 py-1 bg-neutral-200/5 border-neutral-200/25 hover:border-neutral-200/45 hover:bg-neutral-200/15" {
                            "Submit"
                        }
                    }
                }
            }
        ),
    )
    .into_response())
}

#[derive(Deserialize, Debug)]
pub struct IncomingQuote {
    #[serde(rename = "quoteline")]
    lines: Vec<String>,
    #[serde(rename = "quoteauthor")]
    authors: Vec<Uuid>,
    location: String,
    time: String,
    tz_offset: Option<i32>,
    context: String,
}
pub async fn form(
    headers: HeaderMap,
    Form(form): Form<IncomingQuote>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    let authors = form
        .authors
        .into_iter()
        .map(|nid| Name::get_by_id(&tx, nid).unwrap());
    let lines = form.lines.into_iter().zip(authors).collect();
    let offset = form
        .tz_offset
        .and_then(|mins| chrono::FixedOffset::west_opt(mins * 60))
        .unwrap_or_else(|| chrono::FixedOffset::west_opt(0).unwrap());

    let timestamp = chrono::NaiveDateTime::parse_from_str(&form.time, "%Y-%m-%dT%H:%M")
        .map(|ndt| offset.from_local_datetime(&ndt).unwrap())
        .unwrap_or_else(|_| Utc::now().with_timezone(&Warsaw).fixed_offset());
    let context = match form.context.trim() {
        "" => None,
        s => Some(s.to_string()),
    };
    let location = match form.location.trim() {
        "" => None,
        s => Some(s.to_string()),
    };

    let q = Quote::create(&tx, lines, timestamp, context, location, u.id, false)?;
    LogEntry::new(&tx, u, LogAction::CreateQuote { id: q.id })?;
    tx.commit()?;

    Ok(Json(q).into_response())
}
