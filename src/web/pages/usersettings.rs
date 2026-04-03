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
    users::{
        User,
        auth::{AuthError, UserAuthRequired, UserAuthenticate},
        handle::UserHandle,
    },
    web::{components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, AuthError> {
    let u = User::authenticate(req.headers())?;

    Ok(base(
        "Persons | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            @if let Some(u) = u {
                div class="max-w-4xl mx-auto px-2" {
                    div class="mx-auto max-w-4xl my-4" {
                        p class="flex items-center gap-2" {
                            span class="text-neutral-500" {(PreEscaped(icons::SERVER))}
                            span class="text-2xl font-semibold font-lora" {"Your User Settings"}
                        }
                        p class="text-neutral-500 text-sm font-light" {
                            // "Hi, " (u.handle) "!" " " "This is your user settings page." br;
                            "Looking for Mnemosyne settings?" " "
                            a class="text-blue-500 hover:text-blue-400 hover:underline" href="/mnemosyne-settings" {"Here."}
                        }
                    }

                    label for="handle" class="font-light text-neutral-500" {"Handle"}
                    form action="/user-settings/handle" method="post" class="flex gap-2" {
                        div class="flex items-center border border-neutral-200/25 rounded bg-neutral-950/50" {
                            span class="pl-2 text-neutral-500 select-none" {"@"}
                            input id="handle" name="handle" type="text" autocomplete="off" value={(u.handle)}
                                class="w-full bg-transparent pl-0.5 pr-1 py-1 outline-none";
                        }
                        button type="submit" class="px-4 py-1 border border-neutral-200/25 bg-neutral-200/5 rounded cursor-pointer hover:border-neutral-200/40" {
                            "Save"
                        }
                    }
                    hr class="mt-6 mb-4 border-neutral-600";
                    p class="flex items-center gap-1" {
                        span class="text-neutral-500 scale-[.8]" {(PreEscaped(icons::USER_KEY))}
                        span class="text-lg font-semibold font-lora" {"Change Password"}
                    }
                    label for="password" class="font-light text-neutral-500" {"New password"}
                    form action="/user-settings/passwd" method="post" class="flex gap-2" {
                        input id="password" name="password" type="password" autocomplete="off" class="px-2 py-1 border border-neutral-200/25 bg-neutral-950/50 rounded";
                        button type="submit" class="px-4 py-1 border border-neutral-200/25 bg-neutral-200/5 rounded cursor-pointer hover:border-neutral-200/40" {
                            "Submit"
                        }
                    }
                }
            } @else {
                p class="text-center p-2" {"You must be logged in to view this page."}
            }
        ),
    )
    .into_response())
}

#[derive(Deserialize)]
pub struct HandleForm {
    handle: UserHandle,
}
pub async fn change_handle(
    headers: HeaderMap,
    Form(form): Form<HandleForm>,
) -> Result<Response, CompositeError> {
    let mut u = User::authenticate(&headers)?.required()?;
    let oldhandle = u.handle.as_str().to_string();
    u.set_handle(form.handle)?;
    LogEntry::new(
        u.clone(),
        LogAction::ChangeUserHandle {
            id: u.id,
            old: oldhandle,
            new: u.handle.as_str().to_string(),
        },
    )?;
    Ok(Redirect::to("/user-settings").into_response())
}

#[derive(Deserialize)]
pub struct PasswordForm {
    password: String,
}
pub async fn change_password(
    headers: HeaderMap,
    Form(form): Form<PasswordForm>,
) -> Result<Response, CompositeError> {
    let mut u = User::authenticate(&headers)?.required()?;
    u.set_password(Some(&form.password))?;
    Ok(Redirect::to("/user-settings").into_response())
}
