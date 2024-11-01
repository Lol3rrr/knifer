use leptos::*;

use super::*;

#[leptos::component]
pub fn general() -> impl leptos::IntoView {
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
            .map(|team| {
                view! {
                    <TeamScoreboard value=team.players team_name=format!("Team {} - {}", team.number, team.score) />
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <Suspense
            fallback=move || view! { <p>Loading Scoreboard data</p> }
        >
            { scoreboards }
        </Suspense>
    }
}
