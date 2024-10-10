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
            <TeamScoreboard info=scoreboard_resource team_name="Team 1".to_string() part=|s| s.team1 />
            <TeamScoreboard info=scoreboard_resource team_name="Team 2".to_string() part=|s| s.team2 />
        </Suspense>
    }
}

fn damage_sorting(p1: &common::demo_analysis::ScoreBoardPlayer, p2: &common::demo_analysis::ScoreBoardPlayer) -> core::cmp::Ordering {
    p2.damage.cmp(&p1.damage)
}
fn kill_sorting(p1: &common::demo_analysis::ScoreBoardPlayer, p2: &common::demo_analysis::ScoreBoardPlayer) -> core::cmp::Ordering {
    p2.kills.cmp(&p1.kills)
}
fn assists_sorting(p1: &common::demo_analysis::ScoreBoardPlayer, p2: &common::demo_analysis::ScoreBoardPlayer) -> core::cmp::Ordering {
    p2.assists.cmp(&p1.assists)
}
fn deaths_sorting(p1: &common::demo_analysis::ScoreBoardPlayer, p2: &common::demo_analysis::ScoreBoardPlayer) -> core::cmp::Ordering {
    p2.deaths.cmp(&p1.deaths)
}

#[leptos::component]
fn team_scoreboard(info: Resource<leptos_router::ParamsMap, common::demo_analysis::ScoreBoard>, team_name: String, part: fn(common::demo_analysis::ScoreBoard) -> Vec<common::demo_analysis::ScoreBoardPlayer>) -> impl IntoView {
    let (ordering, set_ordering) = create_signal::<fn(&common::demo_analysis::ScoreBoardPlayer, &common::demo_analysis::ScoreBoardPlayer) -> core::cmp::Ordering>(damage_sorting);

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
                    <th on:click=move |_| {
            set_ordering(kill_sorting);
        }>Kills</th>
                    <th on:click=move |_| {
            set_ordering(assists_sorting);
        }>Assists</th>
                    <th on:click=move |_| {
            set_ordering(deaths_sorting);
        }>Deaths</th>
                    <th on:click=move |_| {
            set_ordering(damage_sorting);
        }>Damage</th>
                </tr>
        {
            move || {
                let value = info.get().map(|v| part(v));
                let mut players: Vec<_> = value.into_iter().flat_map(|v| v).collect();
                let sorting = ordering.get();
                players.sort_unstable_by(|p1, p2| sorting(p1, p2));

                players.into_iter().map(|s| {
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
