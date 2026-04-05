use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use maud::{PreEscaped, html};

use crate::{
    database,
    error::CompositeError,
    persons::Name,
    users::{User, auth::UserAuthenticate},
    web::{components::nav::nav, icons, pages::base},
};

pub async fn page(req: Request) -> Result<Response, CompositeError> {
    let u = User::authenticate(req.headers())?;
    let conn = database::conn()?;
    let names = Name::get_all(&conn)?;

    Ok(base(
        "Add Quote | Mnemosyne",
        html!(
            (nav(u.as_ref(), req.uri().path()))

            div class="max-w-4xl mx-auto px-2" {
                div class="my-4 flex justify-between" {
                    p class="flex items-center gap-2" {
                        span class="text-neutral-500" {(PreEscaped(icons::SCROLL_TEXT))}
                        span class="text-2xl font-semibold font-lora" {"Quote Maker"}
                    }
                }
                div class="border border-neutral-200/25 bg-neutral-200/5 rounded-md p-4 flex flex-col" {
                    @for i in 1..=2 {
                        div class="flex justify-between gap-4" {
                            div class="flex flex-col flex-1" {
                                label class="w-full" {
                                    p class="mb-1" {(format!("Quote Line #{i}"))}
                                    input type="text" name="quoteline" placeholder="They said..." autocomplete="off"
                                        class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                                }
                                // label for=(format!("line-{i}")) class="mb-1" {(format!("Quote Line #{i}"))}
                                // input type="text" id=(format!("line-{i}")) name=(format!("line-{i}"))
                                //     placeholder=(format!("They said...")) autocomplete="off"
                                //     class="px-2 py-1 mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                            div class="flex flex-col" {
                                label {
                                    p class="mb-1" {(format!("Quote Author #{i}"))}
                                    select name="quoteauthor" autocomplete="off"
                                        class="px-2 py-1.5 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25"{
                                            option {"--"}
                                            @for name in &names {
                                                option {(name.name)}
                                            }
                                        }
                                }
                                // label for=(format!("who-{i}")) class="mb-1" {(format!("Quote Author #{i}"))}
                                // select id=(format!("line-{i}")) name=(format!("line-{i}")) autocomplete="off"
                                //     class="px-2 py-1.5 mb-2 bg-neutral-950/50 rounded border border-neutral-200/25" {
                                //     option {"--"}
                                //     @for name in &names {
                                //         option {(name.name)}
                                //     }
                                // }
                            }
                        }
                    }
                    hr class="border-neutral-200/25 my-4";
                    div class="flex gap-4 justify-between" {
                        div class="flex flex-col flex-1" {
                            label class="w-full"{
                                p class="mb-1" {"Location"}
                                input type="text" name="location" autocomplete="off" placeholder="Right there!"
                                    class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                        }
                        div class="flex flex-col flex-1" {
                            label class="w-full" {
                                p class="mb-1" {"Time of utterance"}
                                input type="text" name="time" autocomplete="off" placeholder="2026-04-05T01:14:05+02:00"
                                    class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                        }
                    }
                    div class="flex gap-4 justify-between" {
                        div class="flex flex-col flex-1" {
                            label class="w-full" {
                                p class="mb-1" {"Context"}
                                input type="text" name="context" autocomplete="off" placeholder="It was like this.."
                                    class="px-2 py-1 w-full mb-2 bg-neutral-950/50 rounded border border-neutral-200/25";
                            }
                        }
                        button type="submit" class="border mt-auto mb-2 cursor-pointer rounded h-fit px-2 py-1 bg-neutral-200/5 border-neutral-200/25 hover:border-neutral-200/45 hover:bg-neutral-200/15" {
                            "Submit"
                        }
                    }
                }
            }
        ),
    )
    .into_response())
}
