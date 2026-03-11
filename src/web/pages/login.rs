use axum::{
    extract::{Query, Request},
    response::{IntoResponse, Redirect, Response},
};
use maud::{PreEscaped, html};
use rand::seq::IndexedRandom;
use serde::Deserialize;

use crate::{
    config::REFERENCE_SPLASHES,
    users::{
        User,
        auth::{AuthError, UserAuthenticate},
    },
    web::{components::marquee::marquee, icons, pages::base},
};

#[derive(Deserialize)]
pub struct LoginMsg {
    msg: Option<String>,
}

pub async fn page(Query(q): Query<LoginMsg>, req: Request) -> Result<Response, AuthError> {
    let u = User::authenticate(req.headers())?;
    if u.is_some() {
        return Ok(Redirect::to("/dashboard").into_response());
    }
    Ok(base(
        "Log in | Mnemosyne",
        html!(
            div class="min-h-screen flex flex-col items-center overflow-x-hidden" {
                div class="overflow-hidden"{(marquee(&["the goddess of memory"]))}

                div class="mt-24" {}
                a href="/" class="font-semibold text-4xl sm:text-6xl mx-auto font-lora hover:underline" {h1 {"Mnemosyne"}}
                p class="text-neutral-500 mt-4 hidden sm:block text-sm mx-auto" {"The goddess of memory holds all the cards."}
                div class="mb-16" {}

                div class="bg-neutral-200/5 w-4/5 mx-2 sm:mx-0 sm:w-fit border border-neutral-200/25 p-4 rounded" {
                    p class="text-neutral-500 hidden sm:block" {"Part of the olympic pack already? Log in here."}
                    p class="block sm:hidden px-2 w-full text-center text-neutral-500" {"Log in here, olympian."}

                    form id="login-form" method="post" action="/api/auth/login-form" class="mt-8 font-light flex flex-col"  {
                        label for="handle" class="text-neutral-500" {"Handle"}
                        div class="flex items-center w-full border border-neutral-200/25 rounded bg-neutral-950/50" {
                            span class="pl-2 text-neutral-500 select-none" {"@"}
                            input id="handle" name="handle" type="text"
                                class="w-full bg-transparent pl-0.5 pr-1 py-1 outline-none";
                        }

                        label for="password" class="text-neutral-500 font-light mt-2" {"Password"}
                        input id="password" name="password" type="password"
                            class="w-full border border-neutral-200/25 px-2 py-1 rounded bg-neutral-950/50";

                        div class="flex flex-row items-center justify-between mt-4" {
                            @if let Some(msg) = q.msg {
                                p id="login-error" class="text-red-400 text-sm" {
                                    (msg)
                                }
                            } @else {
                                p id="login-error" class="text-red-400 text-sm" {}
                            }
                            button type="submit" class=r#"block ml-auto border border-neutral-200/25 font-normal px-2 py-1
                                rounded bg-neutral-200/5 flex gap-4 justify-between cursor-pointer"# {
                                "Log in"
                                (PreEscaped(icons::ARROW_RIGHT))
                            }
                        }
                        // Logging in is done via JavaScript here by default to preserve
                        // marquee scroll on fail (form-based would reset it on fail)
                        // (if javascript is disabled, login via form still works)
                        script defer {(PreEscaped(r#"
                            if (window.location.search) {
                                history.replaceState(null, '', window.location.pathname);
                            }
                            document.getElementById('login-form').addEventListener('submit', async (e) => {
                                e.preventDefault();
                                const err = document.getElementById('login-error');
                                err.textContent = '';

                                const handle = document.getElementById('handle').value;
                                const password = document.getElementById('password').value;

                                try {
                                    const res = await fetch('/api/auth/login', {
                                        method: 'POST',
                                        headers: { 'Content-Type': 'application/json' },
                                        body: JSON.stringify({ handle, password }),
                                    });

                                    if (res.ok) {
                                        window.location.href = '/dashboard';
                                    } else {
                                        const text = await res.text();
                                        err.textContent = text || 'Login failed';
                                    }
                                } catch (_) {
                                    err.textContent = 'Network error — please try again';
                                }
                            });
                        "#
                        ))}
                    }
                }

                div class="mt-auto" {}
                div class="overflow-hidden"{(marquee(&[REFERENCE_SPLASHES.choose(&mut rand::rng()).unwrap()]))}
            }
        ),
    ).into_response())
}
