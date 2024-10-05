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

    view! {
        <h3>Heatmaps</h3>

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
    }
}

#[leptos::component]
fn heatmap_view(heatmaps: Vec<common::demo_analysis::PlayerHeatmap>) -> impl leptos::IntoView {
    let (idx, set_idx) = create_signal(0usize);
    let (value, set_value) = create_signal(None::<common::demo_analysis::PlayerHeatmap>);

    let h1 = heatmaps.clone();

    let map = use_context::<CurrentDemoName>().unwrap();

    let style = stylers::style! {
        "Heatmap-View",
        .heatmap_image {
            width: 1024px;
            height: 1024px;
            display: block;
            position: relative;
        }
        .heatmap_image > * {
            position: absolute;
        }

        .heatmap_image > .heatmap {
            opacity: 0.5;
        }
    };

    view! {
        class=style,
        <div>
            <select on:change=move |ev| {
                let new_value = event_target_value(&ev);
                let idx: usize = new_value.parse().unwrap();
                set_value(heatmaps.get(idx).cloned());
                set_idx(idx);
            } prop:value=move || idx.get().to_string()>
                { (move |heatmaps: Vec<common::demo_analysis::PlayerHeatmap>| {
            heatmaps.iter().enumerate().map(|(idx, heatmap)| {
                view! {
                    <option value={idx}>{heatmap.name.clone()}</option>
                }
            }).collect::<Vec<_>>()
        })(h1.clone())}
            </select>
            <br />

            {
                    move || {
                        match value.get() {
                            Some(heatmap) => view! {
                                class=style,
                                <div class="heatmap_image">
                                    <img class="radar" src=format!("/static/minimaps/{}.png", map.0.get()) />
                                    <img class="heatmap" width=1024 height=1024 src=format!("data:image/png;base64,{}", heatmap.png_data) />
                                </div>
                            }.into_any(),
                            None => view! { <p>ERROR</p> }.into_any(),
                        }
                    }
                }
        </div>
    }
}
