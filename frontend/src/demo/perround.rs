use leptos::*;

fn to_coloumn(idx: usize) -> usize {
    if idx < 12 {
        2 + idx
    } else if idx < 24 {
        2 + idx + 1
    } else if idx < 27 {
        2 + idx + 2
    } else {
        2 + idx + 3
    }
}

#[leptos::component]
pub fn per_round() -> impl leptos::IntoView {
    let perround_resource =
        create_resource(leptos_router::use_params_map(), |params| async move {
            let id = params.get("id").unwrap();

            let res =
                reqwasm::http::Request::get(&format!("/api/demos/{}/analysis/perround", id))
                    .send()
                    .await
                    .unwrap();
            res.json::<common::demo_analysis::PerRoundResult>()
                .await
                .unwrap()
        });
 
    let style = stylers::style! {
        "PerRound",
        .round_overview {
            display: inline-grid;
            
            width: 90vw;
            grid-template-columns: auto repeat(12, 1fr) 5px repeat(12, 1fr) 5px repeat(3, 1fr) 5px repeat(3, 1fr);
            grid-template-rows: repeat(3, auto);
        }

        .round_entry {
            display: inline-block;

            border-left: 1px solid #101010;
            border-right: 1px solid #101010;

            padding-left: 2px;
            padding-right: 2px;
        }

        .round_number {
            border-top: 1px solid #101010;
            border-bottom: 1px solid #101010;
        }

        p.round_entry {
            margin: 0px;
        }

        .won.ct_won {
            background-color: #1111ff77;
        }
        .won.t_won {
            background-color: #dd111177;
        }
        .lose {
            background-color: #22222277;
        }
    };

    let (round, set_round) = create_signal(0);

    let events_list = move || {
        let round_index = round();
        let current_round = perround_resource.get().map(|rs| rs.rounds.get(round_index).cloned()).flatten();

        match current_round {
            Some(round) => {
                round.events.iter().map(|event| {
                    match event {
                        common::demo_analysis::RoundEvent::BombPlanted => view! { <li>Bomb has been planted</li> }.into_view(),
                        common::demo_analysis::RoundEvent::BombDefused => view! { <li>Bomb has been defused</li> }.into_view(),
                        common::demo_analysis::RoundEvent::Killed { attacker, died } => view! { <li>{"'"}{ attacker }{"'"} killed {"'"}{ died }{"'"}</li> }.into_view(),
                    }
                }).collect::<Vec<_>>().into_view()
            }
            None => view! {}.into_view(),
        }
    };

    let round_overview = move || {
        (0..30).map(|r| {
                let set_round = move |_| {
                    set_round.set(r);
                };

                let round = perround_resource.get().map(|rs| rs.rounds.get(r).cloned()).flatten();
                let reason = round.map(|r| r.reason);

                let (ct_won, t_won) = match &reason {
                    Some(common::demo_analysis::RoundWinReason::TKilled) => (true, false),
                    Some(common::demo_analysis::RoundWinReason::BombDefused) => (true, false),
                    Some(common::demo_analysis::RoundWinReason::TimeRanOut) => (true, false),
                    Some(common::demo_analysis::RoundWinReason::CTKilled) => (false, true),
                    Some(common::demo_analysis::RoundWinReason::BombExploded) => (false, true),
                    _ => (false, false)
                };

                // Upper is CT by default and lower is T by default
                let (mut upper_symbol, mut upper_won) = match &reason {
                    Some(common::demo_analysis::RoundWinReason::TKilled) => (view! { <span>Killed Ts</span> }.into_view(), true),
                    Some(common::demo_analysis::RoundWinReason::BombDefused) => (view! { <span>Defused</span> }.into_view(), true),
                    Some(common::demo_analysis::RoundWinReason::TimeRanOut) => (view! { <span>Out of Time</span> }.into_view(), true),
                    _ => (view! {}.into_view(), false),
                };
                let (mut lower_symbol, mut lower_won) = match &reason {
                    Some(common::demo_analysis::RoundWinReason::CTKilled) => (view! { <span>Killed CTs</span> }.into_view(), true),
                    Some(common::demo_analysis::RoundWinReason::BombExploded) => (view! { <span>Exploded</span> }.into_view(), true),
                    _ => (view! {}.into_view(), false),
                };

                if (12..27).contains(&r) {
                    core::mem::swap(&mut upper_symbol, &mut lower_symbol);
                    core::mem::swap(&mut upper_won, &mut lower_won);
                }

                view! {
                    class=style, 
                    <div 
                        class="round_entry"
                        style=format!("grid-column: {}; grid-row: 1", to_coloumn(r))
                        class:ct_won=move || ct_won
                        class:t_won=move || t_won
                        class:won=move || upper_won
                        class:lose=move || !upper_won
                    >{ upper_symbol } </div>
                    <p on:click=set_round class="round_entry round_number" style=format!("grid-column: {}; grid-row: 2", to_coloumn(r))>{ r + 1 }</p>
                    <div
                        class="round_entry"
                        style=format!("grid-column: {}; grid-row: 3", to_coloumn(r))
                        class:ct_won=move || ct_won
                        class:t_won=move || t_won
                        class:won=move || lower_won
                        class:lose=move || !lower_won
                    > { lower_symbol } </div>
                }
            }).collect::<Vec<_>>()
    };

    let team_names = move || {
        let perround_teams = match perround_resource.get().map(|p| p.teams) {
            Some(t) => t,
            None => return view! {}.into_view(),
        };

        let upper = perround_teams.iter().find(|t| t.name == "CT").map(|t| t.number).unwrap_or(0);
        let lower = perround_teams.iter().find(|t| t.name == "TERRORIST").map(|t| t.number).unwrap_or(0);

        view! {
            <span style="grid-column: 1; grid-row: 1">Team { upper }</span>
            <span style="grid-column: 1; grid-row: 3">Team { lower }</span>
        }.into_view()
    };

    view! {
        class=style,
        <h3>Per Round</h3>

        <div class="round_overview">
            { team_names }
            { round_overview }
        </div>

        <div>
            <h3> Round { move || round.get() + 1 } </h3>
            <div>
                <ul> { events_list } </ul>
            </div>
        </div>
    }
}
