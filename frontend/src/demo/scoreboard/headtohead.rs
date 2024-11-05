use leptos::*;
use leptos::Suspense;

#[leptos::component]
pub fn head_to_head() -> impl leptos::IntoView {
    let head_to_head_resource =
        create_resource(leptos_router::use_params_map(), |params| async move {
            let id = params.get("id").unwrap();

            let res =
                reqwasm::http::Request::get(&format!("/api/demos/{}/analysis/headtohead", id))
                    .send()
                    .await
                    .unwrap();
            res.json::<common::demo_analysis::HeadToHead>()
                .await
                .unwrap()
        });

    let style = stylers::style! {
        "Head-to-Head-root",
        div {
            display: grid;
        }
    };

    let matrix_view = move || head_to_head_resource.get().map(|r| view! {
        <Matrix data=r />
    });

    view! {
        class=style,
        <Suspense fallback=move || view! { <p>Loading Head-to-Head data...</p> }>
            <div>
                { matrix_view }
            </div>
        </Suspense>
    }
}

#[leptos::component]
fn matrix(data: common::demo_analysis::HeadToHead) -> impl leptos::IntoView {
    let row_player_view = move || {
        data.row_players.iter().enumerate().map(|(idx, name)| {
            view! {
                <span style=format!("grid-row: {}; grid-column: 1;", idx + 2)> {name} </span>
            }
        }).collect::<Vec<_>>()
    };

    let column_player_view = move || {
        data.column_players.iter().enumerate().map(|(idx, name)| {
            view! {
                <span style=format!("grid-row: 1; grid-column: {}; text-align: right", idx + 2)> {name} </span>
            }
        }).collect::<Vec<_>>()
    };

    let style = stylers::style! {
        "Head-to-Head-Matrix-Cell",
        .cell {
            display: grid;

        }
        .cell_back {
            grid-row: 1/3;
            grid-column: 1/3;

            background-color: var(--color-surface-a0);
            background-image: linear-gradient(to right top, var(--color-surface-a0) 50%, var(--color-surface-a10) 50%);
        }
    };

    let entry_view = move || {
        data.entries.iter().enumerate().flat_map(|(row, row_data)| {
            row_data.iter().enumerate().map(move |(column, &(row_kills, column_kills))| {
                let entry = move || view! {
                    <div style="grid-row: 2; grid-column: 1; text-align: center;">{ row_kills }</div>
                    <div style="grid-row: 1; grid-column: 2; text-align: center;">{ column_kills }</div>
                };

                view! {
                    class = style,
                    <div class="cell" style=format!("grid-row: {}; grid-column: {};", row + 2, column + 2)>
                        <div class="cell_back"></div>
                        { entry }
                    </div>
                }
            })
        }).collect::<Vec<_>>()
    };

    view! {
        { row_player_view }
        { column_player_view }
        { entry_view }
    }
}
