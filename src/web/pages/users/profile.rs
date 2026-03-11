use axum::{
    extract::{Path, Request},
    response::{IntoResponse, Redirect, Response},
};
use chrono::{DateTime, Utc};
use maud::{PreEscaped, html};
use uuid::Uuid;

use crate::{
    persons::Name,
    quotes::{Quote, QuoteLine},
    users::{
        User, UserError,
        auth::{AuthError, UserAuthenticate},
    },
    web::{
        components::{nav::nav, quote::quote},
        icons,
        pages::base,
    },
};

pub async fn page(Path(id): Path<Uuid>, req: Request) -> Result<Response, AuthError> {
    let u = match User::authenticate(req.headers())? {
        Some(u) => u,
        None => return Ok(Redirect::to("/users").into_response()),
    };
    let user = match User::get_by_id(id) {
        Ok(u) => u,
        Err(UserError::NoUserWithId(_)) => {
            return Ok(base(
                "No such user | Mnemosyne",
                html!(
                    (nav(Some(&u), req.uri().path()))
                    div class="mx-auto max-w-4xl mt-16 text-center" {
                        div class="text-6xl mb-4" { "?" }
                        p class="text-red-400 text-lg" { "No such user found." }
                        p class="text-neutral-500 text-sm mt-2" { "The user you're looking for doesn't exist or has been removed." }
                        a href="/users" class="inline-block mt-6 text-sm bg-neutral-200/5 hover:bg-neutral-200/10 text-neutral-200 border border-neutral-200/25 hover:border-neutral-200/50 rounded px-4 py-2 transition-colors" {
                            "Back to Users"
                        }
                    }
                ),
            )
            .into_response());
        }
        _ => {
            return Ok(base("Error | Mnemosyne", html!(
                (nav(Some(&u), req.uri().path()))
                p class="text-red-400 text-center my-4" { "An error occurred while loading this profile." }
            )).into_response());
        }
    };

    let is_own_profile = u.id == user.id;
    let is_special = user.is_infradmin() || user.is_systemuser();
    let initial = user
        .handle
        .as_str()
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let joined_str = user.created_at().map(|d| d.format("%Y-%m-%d").to_string());
    let sample_quotes = sample_quotes_for_display();

    Ok(base(
        &format!("@{} | Mnemosyne", user.handle),
        html!(
            (nav(Some(&u), req.uri().path()))

            // banner
            div class="relative w-full h-48 sm:h-56 md:h-64 bg-linear-to-b from-neutral-800 from-25% to-emerald-950 overflow-hidden" {
                div class="absolute bottom-0 left-0 right-0 h-px bg-neutral-500/50 --bg-gradient-to-r --from-transparent --via-neutral-500/50 --to-transparent" {}
            }

            div class="mx-auto max-w-4xl px-4 sm:px-6 relative" {
                div class="-mt-16 sm:-mt-20 flex flex-col sm:flex-row sm:items-end gap-4 sm:gap-6" {

                    div class="w-28 h-28 sm:w-36 sm:h-36 rounded-full bg-neutral-800 border-4 border-neutral-900 flex items-center justify-center shadow-xl shrink-0 ring-2 ring-neutral-700/50" {
                        span class="text-4xl sm:text-5xl font-lora font-semibold text-neutral-300 select-none" {(initial)}
                    }

                    div class="flex flex-col gap-1 pb-1" {
                        div class="flex items-center gap-3 flex-wrap" {
                            h1 class="text-2xl md:text-4xl font-semibold font-lora" {
                                (user.handle)
                            }

                            @if is_special {
                                span class="mt-1 inline-flex items-center gap-1 rounded-full bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs px-2.5 py-0.5" {
                                    span class="scale-[.65]"{(PreEscaped(icons::SHIELD_USER))}
                                    "System Account"
                                }
                            }
                            @if is_own_profile {
                                span class="mt-1 inline-flex items-center gap-1 rounded-full bg-neutral-200/5 border border-neutral-200/15 text-neutral-500 text-xs px-2.5 py-0.5" {
                                    span class="scale-[.65]"{(PreEscaped(icons::EYE))}
                                    "This is you"
                                }
                            }
                        }
                    }
                }

                div class="my-6 h-px bg-neutral-200/10" {}

                // about/details
                div class="grid grid-cols-1 md:grid-cols-3 gap-6" {
                    div class="md:col-span-2" {
                        h2 class="text-sm font-semibold text-neutral-400 uppercase tracking-wider mb-3 flex items-center gap-2" {
                            span class="scale-[.7]" {(PreEscaped(icons::INFO))}
                            "About"
                        }
                        div class="border border-neutral-200/15 bg-neutral-200/3 rounded-lg p-4" {
                            @if is_own_profile {
                                p class="text-neutral-500 italic text-sm leading-relaxed" {
                                    "You haven't written a bio yet. Tell people a bit about yourself!"
                                }
                            } @else if is_special {
                                p class="text-neutral-500 italic text-sm leading-relaxed" {
                                    @if user.is_infradmin() {
                                        "The infrastructure administrator account for this Mnemosyne instance."
                                    } @else {
                                        "The internal system account used by Mnemosyne for automated actions."
                                    }
                                }
                            } @else {
                                p class="text-neutral-500 italic text-sm leading-relaxed" {
                                    "This user hasn't written a bio yet."
                                }
                            }
                        }
                    }

                    div class="md:col-span-1" {
                        h2 class="text-sm font-semibold text-neutral-400 uppercase tracking-wider mb-3 flex items-center gap-2" {
                            span class="scale-[.7]" {(PreEscaped(icons::CLIPBOARD_CLOCK))}
                            "Details"
                        }
                        div class="border border-neutral-200/15 bg-neutral-200/3 rounded-lg p-4 space-y-3" {
                            div {
                                p class="text-xs text-neutral-500 uppercase tracking-wide" {"Handle"}
                                p class="text-sm text-neutral-300 mt-0.5" {"@" (user.handle)}
                            }
                            div class="h-px bg-neutral-200/10" {}
                            div {
                                p class="text-xs text-neutral-500 uppercase tracking-wide" {"Member Since"}
                                p class="text-sm text-neutral-300 mt-0.5" {
                                    @if let Some(ref date) = joined_str {(date)}
                                    @else {span class="text-neutral-500" {"Does not apply"}}
                                }
                            }
                            div class="h-px bg-neutral-200/10" {}
                            div {
                                p class="text-xs text-neutral-500 uppercase tracking-wide" {"Account Type"}
                                p class="text-sm text-neutral-300 mt-0.5" {
                                    @if user.is_infradmin() {
                                        "Infrastructure Admin"
                                    } @else if user.is_systemuser() {
                                        "System"
                                    } @else {
                                        "Standard"
                                    }
                                }
                            }
                        }
                    }
                }

                div class="my-6 h-px bg-neutral-200/10" {}

                // quotes-by
                div class="mb-12" {
                    h2 class="text-sm font-semibold text-neutral-400 uppercase tracking-wider mb-1 flex items-center gap-2" {
                        span class="scale-[.7]" {(PreEscaped(icons::SCROLL_TEXT))}
                        "Quotes by " (user.handle)
                    }
                    p class="text-xs text-neutral-500 font-light mb-4" {
                        "Recent quotes added to Mnemosyne by this user."
                    }
                    div class="grid grid-cols-1 ---sm:grid-cols-2 gap-4" {
                        @for q in &sample_quotes {
                            div class="[&>div]:h-full" {(quote(q))}
                        }
                    }
                    @if sample_quotes.is_empty() {
                        div class="border border-neutral-200/10 border-dashed rounded-lg p-8 text-center" {
                            div class="scale-[1.5] text-neutral-700 mx-auto w-fit mb-3" {(PreEscaped(icons::QUOTE))}
                            p class="text-neutral-500 text-sm" {"No quotes found for this user yet."}
                        }
                    }
                }
            }
        ),
    )
    .into_response())
}

fn sample_quotes_for_display() -> Vec<Quote> {
    vec![
        Quote {
            id: Uuid::now_v7(),
            public: true,
            location: Some(String::from("Poznań")),
            context: Some(String::from("Wykład z językoznawstwa")),
            created_by: Uuid::max(),
            timestamp: DateTime::from(Utc::now()),
            lines: vec![
                QuoteLine {
                    id: Uuid::now_v7(),
                    content: String::from("Nie wiem, czy są tutaj osoby fanowskie zipline-ów?"),
                    attribution: Name {
                        id: Uuid::nil(),
                        created_by: Uuid::max(),
                        person_id: Uuid::now_v7(),
                        is_primary: true,
                        name: String::from("dr. Barbara Konat"),
                    },
                },
                QuoteLine {
                    id: Uuid::now_v7(),
                    content: String::from("Taka uprząż co robi pziuuum!"),
                    attribution: Name {
                        id: Uuid::nil(),
                        created_by: Uuid::max(),
                        person_id: Uuid::now_v7(),
                        is_primary: true,
                        name: String::from("dr. Barbara Konat"),
                    },
                },
            ],
        },
        Quote {
            id: Uuid::now_v7(),
            public: true,
            location: Some(String::from("Discord VC")),
            context: Some(String::from("O narysowanej dziewczynie")),
            created_by: Uuid::max(),
            timestamp: DateTime::from(Utc::now()),
            lines: vec![
                QuoteLine {
                    id: Uuid::now_v7(),
                    content: String::from("Czy tu proporcje są zachowane?"),
                    attribution: Name {
                        id: Uuid::now_v7(),
                        created_by: Uuid::max(),
                        person_id: Uuid::now_v7(),
                        is_primary: true,
                        name: String::from("Adam"),
                    },
                },
                QuoteLine {
                    id: Uuid::now_v7(),
                    content: String::from("Adam, ona nie ma kolan."),
                    attribution: Name {
                        id: Uuid::nil(),
                        created_by: Uuid::max(),
                        person_id: Uuid::now_v7(),
                        is_primary: true,
                        name: String::from("Mollin"),
                    },
                },
            ],
        },
    ]
}
