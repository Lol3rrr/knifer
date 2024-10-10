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

    let (ordering, set_ordering) = create_signal::<orderings::Ordering>(orderings::DAMAGE);

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

mod orderings {
    #[derive(Debug, Clone)]
    pub struct Ordering {
        name: SelectedStat,
        pub sort_fn: fn(p1: &common::demo_analysis::ScoreBoardPlayer, p2: &common::demo_analysis::ScoreBoardPlayer) -> core::cmp::Ordering,
    }

    impl Ordering {
        pub fn display_symbol(&self, stat: SelectedStat) -> &'static str {
            if self.name == stat {
                "↑"
            } else {
                "-"
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SelectedStat {
        Damage,
        Kills,
        Deaths,
        Assists,
    }

    pub const DAMAGE: Ordering = Ordering {
        name: SelectedStat::Damage,
        sort_fn: |p1, p2| p2.damage.cmp(&p1.damage),
    };

    pub const KILLS: Ordering = Ordering {
        name: SelectedStat::Kills,
        sort_fn: |p1, p2| p2.kills.cmp(&p1.kills),
    };

    pub const DEATHS: Ordering = Ordering {
        name: SelectedStat::Deaths,
        sort_fn: |p1, p2| p2.deaths.cmp(&p1.deaths),
    };

    pub const ASSISTS: Ordering = Ordering {
        name: SelectedStat::Assists,
        sort_fn: |p1, p2| p2.assists.cmp(&p1.assists),
    };
}

#[leptos::component]
fn team_scoreboard(info: Resource<leptos_router::ParamsMap, common::demo_analysis::ScoreBoard>, team_name: String, part: fn(common::demo_analysis::ScoreBoard) -> Vec<common::demo_analysis::ScoreBoardPlayer>) -> impl IntoView {
    let (ordering, set_ordering) = create_signal::<orderings::Ordering>(orderings::DAMAGE);

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
            set_ordering(orderings::KILLS);
        }>Kills { move || ordering().display_symbol(orderings::SelectedStat::Kills) }</th>
                    <th on:click=move |_| {
            set_ordering(orderings::ASSISTS);
        }>Assists { move || ordering().display_symbol(orderings::SelectedStat::Assists) }</th>
                    <th on:click=move |_| {
            set_ordering(orderings::DEATHS);
        }>Deaths { move || ordering().display_symbol(orderings::SelectedStat::Deaths) }</th>
                    <th on:click=move |_| {
            set_ordering(orderings::DAMAGE);
        }>Damage { move || ordering().display_symbol(orderings::SelectedStat::Damage) }</th>
                </tr>
        {
            move || {
                let value = info.get().map(|v| part(v));
                let mut players: Vec<_> = value.into_iter().flat_map(|v| v).collect();
                let sorting = ordering.get();
                players.sort_unstable_by(|p1, p2| (sorting.sort_fn)(p1, p2));

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
