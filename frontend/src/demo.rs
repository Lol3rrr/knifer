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
    use leptos::Suspense;

    let scoreboard_resource = create_resource(leptos_router::use_params_map(), |params| async move {
        let id = params.get("id").unwrap();

        let res = reqwasm::http::Request::get(&format!("/api/demos/{}/analysis/scoreboard", id))
                .send()
                .await
                .unwrap();
        res.json::<common::demo_analysis::ScoreBoard>().await.unwrap()
    });

    let team_display_func = |team: &[common::demo_analysis::ScoreBoardPlayer]| {
        team.iter().map(|player| {
            view! {
                <tr>
                    <td> { player.name.clone() } </td>
                    <td> { player.kills } </td>
                    <td> { player.deaths } </td>
                </tr>
            }
        }).collect::<Vec<_>>()
    };

    view! {
        <h2>Scoreboard</h2>

        <Suspense
            fallback=move || view! { <p>Loading Scoreboard data</p> }
        >
            <div>
                <h3>Team 1</h3>
                <table>
                    <tr>
                        <th>Name</th>
                        <th>Kills</th>
                        <th>Deaths</th>
                    </tr>
        {
            move || {
                scoreboard_resource.get().map(|s| {
                    let team = s.team1;
                    team_display_func(&team)
                })
            }
        }
                </table>
            </div>

            <div>
                <h3>Team 2</h3>
                <table>
                    <tr>
                        <th>Name</th>
                    </tr>
        {
            move || {
                scoreboard_resource.get().map(|s| {
                    let team = s.team2;
                    team_display_func(&team)
                })
            }
        }
                </table>
            </div>
        </Suspense>
    }
}

#[leptos::component]
pub fn per_round() -> impl leptos::IntoView {
    view! {
        <h3>Per Round</h3>
    }
}
