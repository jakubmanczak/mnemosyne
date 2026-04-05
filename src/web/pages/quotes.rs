use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use maud::{PreEscaped, html};

use crate::{
    error::CompositeError,
    users::{User, auth::UserAuthenticate},
    web::{components::nav::nav, icons, pages::base},
};

pub mod add;

pub async fn page(req: Request) -> Result<Response, CompositeError> {
    let u = User::authenticate(req.headers())?;

    Ok(base(
        "Persons | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            div class="max-w-4xl mx-auto px-2" {
                div class="my-4 flex justify-between" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::SCROLL_TEXT))}
                        span class="text-2xl font-semibold font-lora" {"Quotes"}
                    }
                    @if let Some(_) = u {
                        a href="/quotes/add" class="group border rounded flex items-center gap-1 px-2 border-neutral-200/25 hover:border-neutral-200/45 bg-neutral-400/5 hover:bg-neutral-400/10" {
                            span class="text-neutral-300 group-hover:text-neutral-200" {(PreEscaped(icons::PLUS))}
                            span class="text-neutral-300 group-hover:text-neutral-200" {"Add quote"}
                        }
                    }
                }
                input class="border w-full border-neutral-200/25 hover:border-neutral-200/45 bg-neutral-950/50 p-2 rounded"
                    placeholder="Search for quotes...";
                div class="text-center p-4" {"Search not yet implemented."}
            }
        ),
    )
    .into_response())
}
