use leptos::*;

use super::CurrentDemoName;

#[leptos::component]
pub fn heatmaps() -> impl leptos::IntoView {
    let heatmaps_resource =
        create_resource(leptos_router::use_params_map(), |params| async move {
            let id = params.get("id").unwrap();

            let res =
                reqwasm::http::Request::get(&format!("/api/demos/{}/analysis/heatmap", id))
                    .send()
                    .await
                    .unwrap();
            res.json::<Vec<common::demo_analysis::PlayerHeatmap>>()
                .await
                .unwrap()
        });

    let style = stylers::style! {
        "Heatmap-Wrapper",
        .container {
            margin-top: 1vh;
        }
    };

    view! {
        class=style,
        <div class="container">
            <Suspense fallback=move || view! { <p>Loading Heatmaps</p> }>
                <div>
            {
                move || {
                    heatmaps_resource.get().map(|h| {
                        view! { <HeatmapView heatmaps=h /> }
                    })
                }
            }
                </div>
            </Suspense>
        </div>
    }
}

#[leptos::component]
fn heatmap_view(heatmaps: Vec<common::demo_analysis::PlayerHeatmap>) -> impl leptos::IntoView {
    let (idx, set_idx) = create_signal(0usize);
    let (value, set_value) = create_signal(Vec::<common::demo_analysis::PlayerHeatmap>::new());

    let map = use_context::<CurrentDemoName>().unwrap();

    let style = stylers::style! {
        "Heatmap-View",
        .heatmap_container {
            display: inline-block;
        }
    
        .heatmap_image {
            width: min(40vw, 60vh);
            height: min(40vw, 60vh);
            display: block;
            position: relative;
        }
        .heatmap_image > * {
            position: absolute;
        }

        .heatmap_image > .heatmap {
            opacity: 0.5;
        }

        .heatmap_image > img {
            width: min(40vw, 60vh);
            height: min(40vw, 60vh);
        }

        .player_select {
            width: min(40vw, 60vh);
        }
    };

    let heatmap_view = move || {
        let heatmaps = value.get();
        if heatmaps.is_empty() {
            return Vec::new();
        }

        heatmaps.into_iter().map(|heatmap| {
            view! {
                class=style,
                <div class="heatmap_container">
                    <span>{ heatmap.team }</span>
                    <div class="heatmap_image">
                        <img class="radar" src=format!("/static/minimaps/{}.png", map.0.get()) />
                        <img class="heatmap" src=format!("data:image/png;base64,{}", heatmap.png_data) />
                    </div>
                </div>
            }.into_view()
        }).collect::<Vec<_>>()
    };

    let mut og_players: Vec<_> = heatmaps.iter().map(|h| h.name.clone()).collect();
    og_players.sort_unstable();
    og_players.dedup();

    let players = og_players.clone();
    let select_handler = move |ev| {
        let new_value = event_target_value(&ev);
        let idx: usize = new_value.parse().unwrap();

        let player = players.get(idx).unwrap();

        set_value(heatmaps.iter().filter(|h| &h.name == player).cloned().collect());
        set_idx(idx);
    };

    let players = og_players;
    let select_values = move || {
        players.iter().enumerate().map(|(idx, name)| {
            view! {
                <option value={idx}>{ format!("{}", name) }</option>
            }
        }).collect::<Vec<_>>()
    };

    view! {
        class=style,
        <div>
            <select class="player_select" on:change=select_handler prop:value=move || idx.get().to_string()>
                { select_values }
            </select>
            <br />

            { heatmap_view } 
        </div>
    }
}
