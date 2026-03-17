use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use maud::{PreEscaped, html};

use crate::{
    tags::Tag,
    users::{
        User,
        auth::{AuthError, UserAuthenticate},
    },
    web::{components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, AuthError> {
    let u = User::authenticate(req.headers())?;

    Ok(base(
        "Tags | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            @if let Some(_) = u {
                div class="mx-auto max-w-4xl px-2 my-4" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::TAG))}
                        span class="text-2xl font-semibold font-lora" {"Tags"}
                    }
                    p class="text-neutral-500 text-sm font-light" {
                        @if let Ok(c) = Tag::total_count() {
                            (c) " tags in total."
                        } @else {
                            "Could not get total tag count."
                        }
                    }
                }
                @if let Ok(tags) = Tag::get_all() {
                    div class="max-w-4xl mx-auto mt-4 flex gap-2" {
                        @for tag in &tags {
                            div class="rounded-full px-3 py-1 bg-neutral-200/10 border border-neutral-200/15 flex" {
                                span class="text-neutral-400 text-sm" {"#"}
                                span class="text-sm" {(tag.name)}
                                div class="w-px h-2/3 my-auto mx-2 bg-neutral-200/15" {}
                                div class="text-xs flex items-center" {
                                    "10" span class="*:size-3 ml-1 text-neutral-400" {(PreEscaped(icons::SCROLL_TEXT))}
                                    div class="ml-2" {}
                                    "4" span class="*:size-3 ml-1 text-neutral-400" {(PreEscaped(icons::FILE_IMAGE))}
                                }
                            }
                        }
                    }
                    @if tags.is_empty() {
                        p class="text-center p-2" {"No tags yet. How about making one?"}
                    }
                } @else {
                    p class="text-red-400 text-center" {"Failed to load tags."}
                }
            } @else {
                p class="text-center p-2" {"You must be logged in to view this page."}
            }
        ),
    )
    .into_response())
}
