use maud::{Markup, html};

const SPAN_CLASS: &str = "shrink-0 text-[10px] uppercase tracking-[0.3em] text-neutral-500/40";
const MIN_WORDS: usize = 32;
const COPIES: usize = 4;

pub fn marquee(words: &[&str]) -> Markup {
    let filled = fill_words(words);
    html!(
        div class="overflow-hidden font-lexend font-light select-none border-y border-neutral-500/20 py-3" aria-hidden="true" {
            div class="flex" {
                @for _copy in 0..COPIES {
                    div class="animate-marquee flex shrink-0 gap-8 pr-8" {
                        @for word in &filled {
                            span class=(SPAN_CLASS) { (word) }
                        }
                    }
                }
            }
        }
    )
}

fn fill_words(words: &[&str]) -> Vec<String> {
    if words.is_empty() {
        return Vec::new();
    }
    let reps = (MIN_WORDS.div_ceil(words.len())).max(1);
    words
        .iter()
        .cycle()
        .take(words.len() * reps)
        .map(|w| w.to_string())
        .collect()
}
