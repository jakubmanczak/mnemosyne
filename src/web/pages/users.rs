use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use maud::{PreEscaped, html};

use crate::{
    users::{
        User,
        auth::{AuthError, UserAuthenticate},
    },
    web::{
        components::{nav::nav, user_miniprofile::user_miniprofile},
        icons,
        pages::base,
    },
};

pub async fn page(req: Request) -> Result<Response, AuthError> {
    let u = User::authenticate(req.headers())?;
    let us = match u.is_some() {
        true => User::get_all(),
        false => Ok(vec![]),
    };

    Ok(base(
        "Users | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            @if let Some(_) = u {
                div class="mx-auto max-w-4xl px-2 my-4" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::USERS))}
                        span class="text-2xl font-semibold font-lora" {"Users"}
                    }
                    p class="text-neutral-500 text-sm font-light" {
                        @if let Ok(v) = &us {
                            (v.len()) " users registered with Mnemosyne."
                        }
                    }
                }
                div class="mx-auto max-w-4xl flex flex-wrap gap-4" {
                    @if let Ok(vec) = &us {
                        @for user in vec {
                            (user_miniprofile(user))
                        }
                    } @else {
                        p class="text-center py-4 text-light text-red-500" {"Failed to load users."}
                    }
                }
            } @else {
                p class="text-center p-2" {"You must be logged in to view this page."}
            }
        ),
    )
    .into_response())
}
