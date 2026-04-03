use axum::{
    Form,
    extract::Request,
    http::{HeaderMap, StatusCode},
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
        permissions::Permission,
    },
    web::{components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, AuthError> {
    let u = User::authenticate(req.headers())?;

    Ok(base(
        "Users | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            @if let Some(u) = u {
                div class="mx-auto max-w-4xl px-2 my-4" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::USER_PLUS))}
                        span class="text-2xl font-semibold font-lora" {"Create a new user"}
                    }
                }
                @if let Ok(true) = u.has_permission(Permission::ManuallyCreateUsers) {
                    div class="mx-auto max-w-4xl px-2 mt-4" {
                        form action="/users/create-form" method="post" class="flex flex-col" {
                            label for="handle" class="font-light text-neutral-500" {"Handle"}
                            div class="flex w-64 items-center border border-neutral-200/25 rounded bg-neutral-950/50" {
                                span class="pl-2 text-neutral-500 select-none" {"@"}
                                input id="handle" name="handle" type="text" autocomplete="off"
                                    class="w-fit pl-0.5 pr-1 py-1 outline-none";
                            }
                            label for="password" class="font-light text-neutral-500 mt-4" {"Password"} br;
                            input id="password" name="password" type="password" autocomplete="off"
                                class="px-2 w-64 py-1 border border-neutral-200/25 bg-neutral-950/50 rounded";
                            input type="submit" value="Create"
                                class="px-4 mt-4 w-64 py-1 border border-neutral-200/25 bg-neutral-200/5 rounded cursor-pointer hover:border-neutral-200/40";
                        }
                    }
                } @else {
                    p class="text-center p-2" {"You must have permission to view this page."}
                }
            } @else {
                p class="text-center p-2" {"You must be logged in to view this page."}
            }
        ),
    )
    .into_response())
}

#[derive(Deserialize)]
pub struct CreateUserWithPasswordForm {
    handle: UserHandle,
    password: String,
}
pub async fn create_user(
    headers: HeaderMap,
    Form(form): Form<CreateUserWithPasswordForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    if !u.has_permission(Permission::ManuallyCreateUsers)? {
        return Ok((StatusCode::FORBIDDEN).into_response());
    }
    let mut nu = User::create(form.handle)?;
    nu.set_password(Some(&form.password))?;
    LogEntry::new(
        u,
        LogAction::CreateUser {
            id: nu.id,
            handle: nu.handle.as_str().to_string(),
        },
    )?;
    Ok(Redirect::to("/users").into_response())
}
