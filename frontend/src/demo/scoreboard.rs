use leptos::*;

#[leptos::component]
pub fn scoreboard() -> impl leptos::IntoView {
    use leptos::Suspense;

    let scoreboard_resource =
        create_resource(leptos_router::use_params_map(), |params| async move {
            let id = params.get("id").unwrap();

            let res =
                reqwasm::http::Request::get(&format!("/api/demos/{}/analysis/scoreboard", id))
                    .send()
                    .await
                    .unwrap();
            res.json::<common::demo_analysis::ScoreBoard>()
                .await
                .unwrap()
        });

    view! {
        <h2>Scoreboard</h2>

        <Suspense
            fallback=move || view! { <p>Loading Scoreboard data</p> }
        >
            <TeamScoreboard info=scoreboard_resource team_name="Team 1".to_string() part=|s| s.team1  />
            <TeamScoreboard info=scoreboard_resource team_name="Team 2".to_string() part=|s| s.team2  />
        </Suspense>
    }
}

#[leptos::component]
fn team_scoreboard(info: Resource<leptos_router::ParamsMap, common::demo_analysis::ScoreBoard>, team_name: String, part: fn(common::demo_analysis::ScoreBoard) -> Vec<common::demo_analysis::ScoreBoardPlayer>) -> impl IntoView {
    let style = stylers::style! {
        "Team-Scoreboard",
        tr:nth-child(even) {
            background-color: #dddddd;
        }

        th {
            padding-left: 10px;
            padding-right: 10px;
        }
        th:nth-child(1) {
            width: 200px;
        }
    };

    view! {
        class = style,
        <div>
            <h3>{ team_name }</h3>
            <table>
                <tr>
                    <th>Name</th>
                    <th>Kills</th>
                    <th>Assists</th>
                    <th>Deaths</th>
                    <th>Damage</th>
                </tr>
        {
            move || {
                let value = info.get().map(|v| part(v));
                (value).into_iter().flat_map(|v| v).map(|s| {
                    view! {
                        class=style,
                        <tr><td>{ s.name }</td><td>{ s.kills }</td><td>{ s.assists }</td><td>{ s.deaths }</td><td>{ s.damage }</td></tr>
                    }
                }).collect::<Vec<_>>()
            }
        }
                </table>
            </div>
    }
}
