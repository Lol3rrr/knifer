use leptos::*;
use leptos_router::{Outlet, A};

#[leptos::component]
pub fn demo() -> impl leptos::IntoView {
    let params = leptos_router::use_params_map();
    let id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    let demo_info = create_resource(
        || (),
        move |_| async move {
            let res = reqwasm::http::Request::get(&format!("/api/demos/{}/info", id()))
                .send()
                .await
                .unwrap();
            res.json::<common::DemoInfo>().await.unwrap()
        },
    );

    let map = move || match demo_info.get() {
        Some(v) => v.map.clone(),
        None => String::new(),
    };

    let selected_tab = move || {
        let loc = leptos_router::use_location();
        let loc_path = loc.pathname.get();
        let trailing = loc_path.split('/').last();
        trailing.unwrap_or("/").to_owned()
    };

    let style = stylers::style! {
        "Demo",
        .analysis_bar {
            display: grid;
            grid-template-columns: auto auto;
            column-gap: 20px;

            background-color: #2d2d2d;
        }

        .analysis_selector {
            display: inline-block;
        }

        span {
            display: inline-block;

            padding: 1vw 1vh;
            color: #d5d5d5;
            background-color: #4d4d4d;
        }
        .current {
            background-color: #5d5d5d;
        }
    };

    view! {class = style,
        <h2>Demo - { id } - { map }</h2>

        <div class="analysis_bar">
            <div class="analysis_selector" class:current=move || selected_tab() == "scoreboard"><A href="scoreboard"><span>Scoreboard</span></A></div>
            <div class="analysis_selector" class:current=move || selected_tab() == "perround"><A href="perround"><span>Per Round</span></A></div>
        </div>
        <div>
            <Outlet/>
        </div>
    }
}

#[leptos::component]
pub fn scoreboard() -> impl leptos::IntoView {
    view! {
        <h2>Scoreboard</h2>

        <div>
            <h3>Team 1</h3>
            <table>
                <tr>
                    <th>Name</th>
                </tr>
            </table>
        </div>

        <div>
            <h3>Team 2</h3>
            <table>
                <tr>
                    <th>Name</th>
                </tr>
            </table>
        </div>
    }
}

#[leptos::component]
pub fn per_round() -> impl leptos::IntoView {
    view! {
        <h3>Per Round</h3>
    }
}
