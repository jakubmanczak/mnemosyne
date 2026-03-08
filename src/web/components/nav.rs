use maud::{Markup, PreEscaped, html};

use crate::{users::User, web::icons};

const LINKS: &[(&str, &str, &str)] = &[
    ("Dashboard", "/dashboard", icons::LAYOUT_DASHBOARD),
    ("Quotes", "#quotes", icons::SCROLL_TEXT),
    ("Photos", "#photos", icons::FILE_IMAGE),
    ("Persons", "#persons", icons::CONTACT),
    ("Tags", "#tags", icons::TAG),
    ("Users", "#users", icons::USERS),
    ("Logs", "#logs", icons::CLIPBOARD_CLOCK),
];

pub fn nav(user: Option<User>, uri: &str) -> Markup {
    html!(
        div class="flex items-center text-sm gap-4 border-b border-neutral-200/25 bg-neutral-200/5 px-4 py-2" {
            a href="/dashboard" class="font-lora font-semibold text-xl mr-2" {"Mnemosyne"}
            div class="w-px h-5 bg-neutral-200/15" {}
            div class="flex flex-row" {
                @for link in LINKS {
                    a href={(link.1)} class="flex flex-row px-2 py-1 rounded items-center gap-2 hover:bg-neutral-200/5 border border-transparent hover:border-neutral-200/25" {
                        @if uri.starts_with(link.1) {
                            div class="scale-[.75] text-neutral-300" {(PreEscaped(link.2))}
                            span class="text-neutral-200 font-light" { (link.0) }
                        } @else {
                            div class="scale-[.75] text-neutral-500" {(PreEscaped(link.2))}
                            span class="text-neutral-400 font-light" { (link.0) }
                        }
                    }
                }
            }


            @if let Some(u) = user {
                a href="/dashboard" class=r#"ml-auto bg-neutral-200/5 font-lexend flex
                    flex-row items-center border border-neutral-200/25 gap-4 rounded px-2 py-1"# {
                    (u.handle)
                    div class="scale-[.75]" {(PreEscaped(icons::USER))}
                }
            } @else {
                a href="/login" class=r#"ml-auto bg-neutral-200/5 font-lexend flex
                    flex-row items-center border border-neutral-200/25 gap-4 rounded px-2 py-1"# {
                    "Log in"
                    div class="scale-[.75]" {(PreEscaped(icons::USER_KEY))}
                }
            }
        }
    )
}
