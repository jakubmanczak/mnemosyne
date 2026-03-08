use maud::{Markup, PreEscaped, html};

use crate::{users::User, web::icons};

pub fn user_miniprofile(u: &User) -> Markup {
    let show_shield = u.is_infradmin() || u.is_systemuser();
    html!(
        a href=(format!("/users/{}", u.id))
            class="w-70 border border-neutral-200/25 hover:border-neutral-200/50 bg-neutral-200/5 hover:bg-neutral-200/10 transition-colors rounded flex" {
            div class="bg-neutral-200/10 text-neutral-300 font-semibold aspect-square flex items-center justify-center" {
                (u.handle.as_str().chars().next().unwrap_or('?').to_uppercase())
            }
            div class="p-3" {
                p class="text-semibold flex" {
                    (u.handle)
                    @if show_shield {
                        span class="scale-[.75] text-neutral-500"
                            title="This is a special internal user." {(PreEscaped(icons::SHIELD_USER))}
                    }
                }
                p class="text-xs text-neutral-500 flex items-center mt-1" {
                    @if show_shield {
                        span class="scale-[.5] -ml-1" {(PreEscaped(icons::SERVER))}
                        "System account"
                    } @else {
                        span class="scale-[.5] -ml-1" {(PreEscaped(icons::CALENDAR_1))}
                        (u.created_at().map_or("Unknown".into(), |d| d.to_string()))
                    }
                }
            }
        }
    )
}
