use axum::extract::Request;
use maud::{Markup, html};

use crate::{
    users::{User, auth::UserAuthenticate},
    web::{components::nav::nav, pages::base},
};

pub async fn page(req: Request) -> Markup {
    let u = User::authenticate(req.headers()).ok().flatten();
    base(
        "Dashboard | Mnemosyne",
        html!(
            (nav(u, req.uri().path()))

            div class="text-8xl text-neutral-800/25 mt-16 text-center font-semibold font-lora select-none" {"Mnemosyne"}
        ),
    )
}
