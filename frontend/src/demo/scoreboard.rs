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

    let scoreboards = move || {
        scoreboard_resource
            .get()
            .into_iter()
            .flat_map(|v| v.teams.into_iter())
            .map(|(team, players)| {
                view! {
                    <TeamScoreboard value=players team_name=format!("Team {}", team) />
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <h2>Scoreboard</h2>

        <Suspense
            fallback=move || view! { <p>Loading Scoreboard data</p> }
        >
            { scoreboards }
        </Suspense>
    }
}

mod orderings {
    #[derive(Debug, Clone)]
    pub struct Ordering {
        name: SelectedStat,
        pub sort_fn: fn(
            p1: &common::demo_analysis::ScoreBoardPlayer,
            p2: &common::demo_analysis::ScoreBoardPlayer,
        ) -> core::cmp::Ordering,
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
fn team_scoreboard(
    value: Vec<common::demo_analysis::ScoreBoardPlayer>,
    team_name: String,
) -> impl IntoView {
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
                let mut players: Vec<_> = value.clone().into_iter().collect();
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
