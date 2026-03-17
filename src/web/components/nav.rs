use maud::{Markup, PreEscaped, html};

use crate::{users::User, web::icons};

// (SHOWTEXT, LINK, ICON, REQUIRES_LOG_IN)
const LINKS: &[(&str, &str, &str, bool)] = &[
    ("Dashboard", "/dashboard", icons::LAYOUT_DASHBOARD, false),
    ("Quotes", "#quotes", icons::SCROLL_TEXT, false),
    ("Photos", "#photos", icons::FILE_IMAGE, false),
    ("Persons", "#persons", icons::CONTACT, false),
    ("Tags", "/tags", icons::TAG, false),
    ("Users", "/users", icons::USERS, true),
    ("Logs", "#logs", icons::CLIPBOARD_CLOCK, true),
];

pub fn nav(user: Option<&User>, uri: &str) -> Markup {
    html!(
        div class="flex items-center text-sm gap-4 border-b border-neutral-200/25 bg-neutral-200/5 px-4 py-2" {
            a href="/dashboard" class="font-lora font-semibold hidden xs:block md:text-xl sm:mr-2" {"Mnemosyne"}
            div class="w-px h-5 bg-neutral-200/15 hidden sm:block" {}
            div class="flex flex-row" {
                @for link in LINKS {
                    @if !link.3 || user.is_some() {
                        a href={(link.1)} class="flex flex-row px-2 py-1 rounded items-center gap-2 hover:bg-neutral-200/5 border border-transparent hover:border-neutral-200/25" {
                            @if uri.starts_with(link.1) {
                                div class="scale-[.75] text-neutral-300" {(PreEscaped(link.2))}
                                span class="text-neutral-200 font-light hidden lg:block" { (link.0) }
                            } @else {
                                div class="scale-[.75] text-neutral-500" {(PreEscaped(link.2))}
                                span class="text-neutral-400 font-light hidden lg:block" { (link.0) }
                            }
                        }
                    }
                }
            }


            @if let Some(u) = user {
                div class="ml-auto relative group" tabindex="0" id="user-dropdown-menu"
                    aria-haspopup="menu"
                    onkeydown="if(event.key === 'Enter' || event.key === ' '){ event.preventDefault(); if(this.matches(':focus-within')){this.blur()}else{this.focus()} }" {
                    div class=r#"bg-neutral-200/5 font-lexend flex
                        flex-row items-center border border-neutral-200/25 gap-4 rounded px-2 py-1 cursor-pointer"#
                        onmousedown="event.preventDefault(); if(this.parentElement.matches(':focus-within')){this.parentElement.blur()}else{this.parentElement.focus()}"
                        ontouchstart="event.preventDefault(); if(this.parentElement.matches(':focus-within')){this.parentElement.blur()}else{this.parentElement.focus()}" {
                        span class="hidden sm:block"{(u.handle)}
                        div class="scale-[.75]" {(PreEscaped(icons::USER))}
                    }
                    div class="absolute right-0 top-full pt-1 w-40 opacity-0 invisible group-focus-within:opacity-100 group-focus-within:visible transition-all duration-100 z-50" {
                        div class="rounded bg-neutral-900 border border-neutral-200/25 shadow-lg flex flex-col overflow-hidden" {
                            a href=(format!("/users/{}", u.id)) class="px-4 py-2 flex items-center gap-2 hover:bg-neutral-200/10 font-lexend text-sm text-neutral-200 transition-colors" {
                                div class="scale-[.7]" {(PreEscaped(icons::USER))}
                                p {"Profile"}
                            }
                            a href="/user-settings" class="px-4 py-2 flex items-center gap-2 hover:bg-neutral-200/10 font-lexend text-sm text-neutral-200 transition-colors" {
                                div class="scale-[.7]" {(PreEscaped(icons::SERVER))}
                                p {"Settings"}
                            }
                            div class="h-px w-full bg-neutral-200/15" {}
                            a href="/api/auth/logout-form" class="w-full text-left flex items-center gap-2 px-4 py-2 hover:bg-neutral-200/10 font-lexend text-sm text-red-300 transition-colors" {
                                div class="scale-[.7]" {(PreEscaped(icons::LOG_OUT))}
                                p {"Log out"}
                            }
                        }
                    }
                }
            } @else {
                a href="/login" class=r#"ml-auto bg-neutral-200/5 font-lexend flex
                    flex-row items-center border border-neutral-200/25 gap-4 rounded px-2 py-1"# {
                    span class="hidden sm:block"{"Log in"}
                    div class="scale-[.75]" {(PreEscaped(icons::USER_KEY))}
                }
            }
        }
        script {
            (PreEscaped(r#"
                document.addEventListener('touchstart', function(e) {
                    const menu = document.getElementById('user-dropdown-menu');
                    if (menu && !menu.contains(e.target) && menu.contains(document.activeElement)) {
                        document.activeElement.blur();
                    }
                }, {passive: true});
                document.addEventListener('keydown', function(e) {
                    if (e.key === 'Escape') {
                        const menu = document.getElementById('user-dropdown-menu');
                        if (menu && menu.contains(document.activeElement)) {
                            document.activeElement.blur();
                        }
                    }
                });
            "#))
        }
    )
}
