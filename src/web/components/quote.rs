use maud::{Markup, PreEscaped, html};

use crate::{quotes::Quote, web::icons};

pub fn quote(quote: &Quote) -> Markup {
    html!(
        div class="border border-neutral-200/25 bg-neutral-200/5 p-3 pb-1 overflow-clip rounded-md relative flex flex-col" {
            div class="absolute top-4 right-6 -rotate-12 opacity-[.025] scale-x-[4.5] scale-y-[4]" {
                (PreEscaped(icons::QUOTE))
            }
            @for (i, line) in quote.lines.iter().enumerate() {
                @let show_author = i == quote.lines.len()-1 || quote.lines[i+1].attribution.id != line.attribution.id;
                div class="mb-2" {
                    span class="flex flex-row gap-2 relative" {
                        span class="scale-x-[.65] scale-y-[.5] absolute opacity-[.3]"{
                            (PreEscaped(icons::QUOTE))
                        }
                        p class="font-lora ml-6"{(line.content)}
                    }
                    @if show_author {
                        p class="text-sm italic ml-3 flex flex-row gap-[6px] text-neutral-400" {
                            "— " (line.attribution.name)
                        }
                    }
                }
            }
            div class="flex flex-row text-neutral-400 mt-auto pt-4 text-sm items-center font-light text-xs" {
                p {(quote.timestamp.format("%d/%m/%Y %H:%M"))}
                @if let Some(loc) = &quote.location {
                    span class="ml-3 scale-[.5]"{(PreEscaped(icons::MAP_PIN))} p { (loc) }
                }
                @if let Some(ctx) = &quote.context {
                    span class="ml-3 scale-[.5]"{(PreEscaped(icons::INFO))} p class="italic truncate pr-1" {(ctx)}
                }
            }
        }
    )
}
