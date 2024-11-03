use leptos::*;
use leptos_router::{Outlet, A};

pub mod heatmap;
pub mod perround;
pub mod scoreboard;

#[derive(Debug, Clone)]
struct CurrentDemoName(ReadSignal<String>);

#[leptos::component]
pub fn demo() -> impl leptos::IntoView {
    let params = leptos_router::use_params_map();
    let id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    let (rx, map_tx) = create_signal(String::new());
    provide_context(CurrentDemoName(rx.clone()));

    let demo_info = create_resource(
        || (),
        move |_| async move {
            let res = reqwasm::http::Request::get(&format!("/api/demos/{}/info", id()))
                .send()
                .await
                .unwrap();
            let value = res.json::<common::DemoInfo>().await.unwrap();

            map_tx.set(value.map.clone());

            value
        },
    );

    let rerun_analysis = create_action(move |_: &()| async move {
        let _ = reqwasm::http::Request::get(&format!("/api/demos/{}/reanalyse", id()))
            .send()
            .await;
    });

    let map = move || match demo_info.get() {
        Some(v) => v.map.clone(),
        None => String::new(),
    };

    let style = stylers::style! {
        "Demo",
        .analysis_bar {
            display: grid;
            grid-template-columns: auto auto auto;
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

        <button on:click=move |_| rerun_analysis.dispatch(())>Rerun analysis</button>
        <TabBar prefix=move || format!("/demo/{}/", id()) parts=&[("scoreboard", "Scoreboard"), ("perround", "Per Round"), ("heatmaps", "Heatmaps")] />
        <div>
            <Outlet/>
        </div>
    }
}

#[leptos::component]
pub fn tab_bar<P>(
    prefix: P,
    parts: &'static [(&'static str, &'static str)],
) -> impl leptos::IntoView
where
    P: Fn() -> String + Copy + 'static,
{
    let selected_tab = move || {
        let prefix = prefix();
        let loc = leptos_router::use_location();
        let loc_path = loc.pathname.get();
        let trailing = loc_path
            .strip_prefix(&prefix)
            .unwrap_or(&loc_path)
            .split('/')
            .filter(|l| !l.is_empty())
            .next();
        trailing
            .or(parts.first().map(|p| p.0))
            .unwrap_or("")
            .to_owned()
    };

    let style = stylers::style! {
        "Demo",
        .analysis_bar {
            display: grid;
            grid-template-columns: repeat(var(--rows), auto);
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

    let tabs = move || {
        parts.into_iter().map(|(routename, name)| {
        view! {class=style,
            <div class="analysis_selector" class:current=move || selected_tab() == routename.to_string()>
                <A href=routename.to_string()>
                    <span>{ name.to_string() }</span>
                </A>
            </div>
        }
    }).collect::<Vec<_>>()
    };

    view! {class = style,
        <div class="analysis_bar" style=format!("--rows: {}", parts.len())>
            { tabs }
        </div>
    }
}
