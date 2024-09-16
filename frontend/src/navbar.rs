use leptos::*;
use leptos_router::A;

use crate::DemoUploadStatus;

#[leptos::component]
fn steam_login(height: &'static str, width: &'static str) -> impl leptos::IntoView {
    let user_status = create_resource(
        || (),
        |_| async move {
            let res = reqwasm::http::Request::get("/api/user/status")
                .send()
                .await
                .unwrap();
            res.json::<common::UserStatus>().await.map_err(|e| ())
        },
    );

    let tmp = move || {
        match user_status.get() {
        Some(Ok(user)) => 
            view! {
                <p>{user.name}</p>
            }.into_view(),
        _ => 
        view! {    
            <a href="/api/steam/login" rel="external">
                <img src="https://community.akamai.steamstatic.com/public/images/signinthroughsteam/sits_01.png" alt="Steam Login" style=format!("height: {height}; width: {width}") />
            </a>
        }.into_view(),
    }
    };

    view! {
        { tmp }
    }
}

#[leptos::component]
pub fn top_bar(update_demo_visible: WriteSignal<DemoUploadStatus>) -> impl leptos::IntoView {
    let style = stylers::style! {
        "TopBar",
        .bar {
            width: 100%;
            height: 4vh;
            padding-top: 0.5vh;
            padding-bottom: 0.5vh;

            background-color: #28282f;
            color: #d5d5d5;

            display: grid;
            grid-template-columns: 15vw auto 10vw calc(4vh * (180/35) + 20px);
        }

        .elem {
            display: inline-block;
            margin-top: auto;
            margin-bottom: auto;
        }

        .logo {
            color: #d5d5d5;
            width: 15vw;
            font-size: 24px;
            padding: 0px;
            margin: 0px;
            margin-left: 1vw;
        }
    };

    view! {class = style,
        <div class="bar">
            <A href="/">
                <p class="logo">Knifer</p>
            </A>

            <div class="elem" style="grid-column-start: 3">
                <button on:click=move |_| update_demo_visible(DemoUploadStatus::Shown)>
                    Upload Demo
                </button>
            </div>

            <div class="elem" style="grid-column-start: 4">
                <SteamLogin height="4vh" width="auto" />
            </div>
        </div>
    }
}
