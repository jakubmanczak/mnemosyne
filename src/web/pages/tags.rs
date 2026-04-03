use axum::{
    Form,
    extract::Request,
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use maud::{PreEscaped, html};
use serde::Deserialize;

use crate::{
    api::CompositeError,
    logs::{LogAction, LogEntry},
    tags::{Tag, TagName},
    users::{
        User,
        auth::{AuthError, UserAuthRequired, UserAuthenticate},
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
                    div class="max-w-4xl mx-auto mt-4 flex flex-wrap gap-2" {
                        @for tag in &tags {
                            div class="rounded-full px-3 py-1 bg-neutral-200/10 border border-neutral-200/15 flex" {
                                span class="text-neutral-400 text-sm" {"#"}
                                span class="text-sm" {(tag.name)}
                                div class="w-px h-2/3 my-auto mx-2 bg-neutral-200/15" {}
                                div class="text-xs flex items-center" {
                                    (
                                        if let Ok(i) = tag.get_tagged_quotes_count() {
                                            i.to_string()
                                        } else {
                                            "?".to_string()
                                        }
                                    ) span class="*:size-3 ml-1 text-neutral-400" {(PreEscaped(icons::SCROLL_TEXT))}
                                    // div class="ml-2" {}
                                    // "0" span class="*:size-3 ml-1 text-neutral-400" {(PreEscaped(icons::FILE_IMAGE))}
                                }
                            }
                        }
                    }
                    @if tags.is_empty() {
                        p class="text-center p-2" {"No tags yet. How about making one?"}
                    }
                    div class="mx-auto max-w-4xl mt-4 px-2" {
                        h3 class="font-lora font-semibold text-xl" {"Add new tag"}
                        form action="/tags/create" method="post" {
                            label for="tagname" class="text-neutral-500 font-light mt-2" {"Tag Name"}
                            div class="flex gap-2" {
                                input type="text" autocomplete="off" id="tagname" name="tagname" placeholder="e.g. fashion"
                                class="px-2 py-1 border border-neutral-200/25 bg-neutral-950/50 rounded";
                                button type="submit"
                                    class="px-4 py-1 border border-neutral-200/25 bg-neutral-200/5 rounded cursor-pointer hover:border-neutral-200/40" {"Submit"}
                            }
                        }
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

#[derive(Deserialize)]
pub struct TagForm {
    tagname: TagName,
}
pub async fn create(
    headers: HeaderMap,
    Form(form): Form<TagForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let t = Tag::create(form.tagname)?;
    LogEntry::new(
        u,
        LogAction::CreateTag {
            id: t.id,
            name: t.name.to_string(),
        },
    )?;
    Ok(Redirect::to("/tags").into_response())
}
