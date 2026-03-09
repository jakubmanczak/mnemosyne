use maud::{Markup, html};

pub mod marquee;
pub mod nav;
pub mod quote;
pub mod user_miniprofile;

pub fn chip(inner: Markup) -> Markup {
    html!(
        div class="rounded-full px-3 py-1 bg-neutral-200/10 border border-neutral-200/15 text-xs" {
            (inner)
        }
    )
}
