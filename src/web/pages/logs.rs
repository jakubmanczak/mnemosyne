use axum::{
    extract::Request,
    response::{IntoResponse, Redirect, Response},
};
use maud::{PreEscaped, html};

use crate::{
    api::CompositeError,
    logs::LogEntry,
    users::{User, auth::UserAuthenticate, permissions::Permission},
    web::{RedirectViaError, components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, CompositeError> {
    let u = User::authenticate(req.headers())?
        .ok_or(RedirectViaError(Redirect::to("/login?re=/logs")))?;
    let logs = LogEntry::get_all()?;

    Ok(base(
        "Persons | Mnemosyne",
        html!(
            (nav(Some(&u), req.uri().path()))

            @if let Ok(true) = u.has_permission(Permission::BrowseServerLogs) {
                div class="max-w-4xl mx-auto px-2" {
                    div class="my-4" {
                        p class="flex items-center gap-2" {
                            span class="text-neutral-500" {(PreEscaped(icons::CLIPBOARD_CLOCK))}
                            span class="text-2xl font-semibold font-lora" {"Logs"}
                        }
                        p class="text-neutral-500 text-sm font-light" {
                            "Work in progress."
                        }
                    }
                    div class="w-full border border-neutral-200/25 rounded grid grid-cols-[auto_auto_1fr]" {
                        @for (txt, ico) in [("Timestamp", icons::CLOCK), ("Actor", icons::USER), ("Action", icons::PEN)] {
                            div class="p-2 flex gap-1 font-semibold border-b border-neutral-200/25" {
                                span class="text-neutral-500 scale-[.8]" {(PreEscaped(ico))}
                                (txt)
                            }
                        }
                        @for (idx, log) in logs.iter().enumerate() {
                            @let s = if idx % 2 == 0 {"background-color: #e5e5e50b"} else {""};
                            div class="p-2 font-light" style=(s) {
                                (log.id.get_timestamp()
                                    .map(|ts| {
                                        let (secs, nanos) = ts.to_unix();
                                        chrono::DateTime::from_timestamp(secs as i64, nanos)
                                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                            .unwrap_or_else(|| "invalid date".to_string())
                                    })
                                    .unwrap_or_else(|| "no timestamp".to_string()))
                            }
                            div class="p-2 font-light" style=(s) {(log.actor.handle)}
                            div class="p-2 font-light" style=(s) {(log.data.get_humanreadable_payload())}
                        }
                        @if true {
                            div class="p-2 col-span-3 text-center font-light text-neutral-400" {"You've reached the end of all logs."}
                        }
                    }
                }
            } @else {
                p class="text-center p-2" {"You must have permission to view this page."}
            }
        ),
    )
    .into_response())
}
