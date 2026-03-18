use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use maud::{PreEscaped, html};

use crate::{
    persons::Person,
    users::{
        User,
        auth::{AuthError, UserAuthenticate},
    },
    web::{components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, AuthError> {
    let u = User::authenticate(req.headers())?;

    Ok(base(
        "Persons | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            @if let Some(_) = u {
                div class="mx-auto max-w-4xl px-2 my-4" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::CONTACT))}
                        span class="text-2xl font-semibold font-lora" {"Persons"}
                    }
                    p class="text-neutral-500 text-sm font-light" {
                        @if let Ok(c) = Person::total_count() {
                            (c) " persons in total."
                        } @else {
                            "Could not get total person count."
                        }
                    }
                }
                @if let Ok(persons) = Person::get_all() {
                    div class="max-w-4xl mx-auto mt-4 flex gap-2" {
                        @for person in &persons {
                            div class="rounded px-4 py-2 bg-neutral-200/10 border border-neutral-200/15 flex items-center" {
                                span class="text-neutral-400 mr-1" {"~"}
                                span class="text-sm" {(person.primary_name)}
                                div class="w-px h-2/3 my-auto mx-2 bg-neutral-200/15" {}
                                div class="text-xs flex items-center" {
                                    (
                                        if let Ok(i) = person.get_in_quote_count() {
                                            i.to_string()
                                        } else {
                                            "?".to_string()
                                        }
                                    ) span class="*:size-3 ml-1 text-neutral-400" {(PreEscaped(icons::SCROLL_TEXT))}
                                    // div class="ml-2" {}
                                    // "4" span class="*:size-3 ml-1 text-neutral-400" {(PreEscaped(icons::FILE_IMAGE))}
                                }
                            }
                        }
                    }
                    @if persons.is_empty() {
                        p class="text-center p-2" {"No persons yet."}
                    }
                } @else {
                    p class="text-red-400 text-center" {"Failed to load persons."}
                }
            } @else {
                p class="text-center p-2" {"You must be logged in to view this page."}
            }
        ),
    )
    .into_response())
}
