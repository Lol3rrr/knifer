use leptos::*;

#[leptos::component]
pub fn homepage(get_notification: ReadSignal<u8>) -> impl leptos::IntoView {
    let demo_data = create_resource(
        move || get_notification.get(),
        |_| async move {
            let res = reqwasm::http::Request::get("/api/demos/list")
                .send()
                .await
                .unwrap();
            let demos: common::DemoList = res.json().await.unwrap();
            demos
        },
    );

    let pending_display = move || {
        demo_data
            .get()
            .map(|d| d.pending)
            .filter(|p| p.len() > 0)
            .map(|pending| {
                view! {
                    <p>{pending.len()} demos are pending/waiting for analysis</p>
                }
            })
    };

    view! {
        <div>
            <div>
                <h2>Demos</h2>
            </div>
            { pending_display }
            <DemoList demos=demo_data />
        </div>
    }
}

#[leptos::component]
fn demo_list(
    demos: impl SignalGet<Value = Option<common::DemoList>> + 'static,
) -> impl leptos::IntoView {
    let style = stylers::style! {
        "DemoList",
        .list {
            display: inline-grid;

            grid-template-columns: auto auto auto;
            row-gap: 1ch;
        }
    };

    view! {
        class=style,
        <div class="list">
            <span>Score</span>
            <span>Date</span>
            <span>Map</span>

            { move || demos.get().map(|d| d.done).unwrap_or_default().into_iter().enumerate().map(|(i, demo)| view! { <DemoListEntry demo idx=i+1 /> }).collect::<Vec<_>>() }
        </div>
    }
}

#[leptos::component]
fn demo_list_entry(demo: common::BaseDemoInfo, idx: usize) -> impl leptos::IntoView {
    let style = stylers::style! {
        "DemoListEntry",
        .entry {
            display: inline-block;

            border: solid #030303aa 1px;

            grid-column: 1 / 4;
            width: 100%;
            height: 100%;
        }

        .score, .map {
            padding-left: 5px;
            padding-right: 5px;

            margin-top: auto;
            margin-bottom: auto;
        }
        .score {
            grid-column: 1;
            font-size: 20px;
        }
        .date {
            grid-column: 2;
        }
        .map {
            grid-column: 3;
        }

        .date {
            display: inline-grid;

            grid-template-columns: auto;
            grid-template-rows: auto auto;
        }

        .won {
            color: #00aa00;
        }
        .lost {
            color: #aa0000;
        }
    };

    let (player_score, enemy_score) = if demo.player_team == 2 {
        (demo.team2_score, demo.team3_score)
    } else {
        (demo.team3_score, demo.team2_score)
    };
    let won = move || player_score > enemy_score;
    let lost = move || enemy_score > player_score;
    let tie = move || player_score == enemy_score;

    view! {
        class=style,
            <span
                class="score"
                style=format!("grid-row: {};", idx + 1)
                class:won=won
                class:lost=lost
                class:tie=tie
            >{demo.team2_score}:{demo.team3_score}</span>
            <div class="date" style=format!("grid-row: {};", idx + 1)>
                <span>{demo.uploaded_at.format("%Y-%m-%d").to_string()}</span>
                <span>{demo.uploaded_at.format("%H-%M-%S").to_string()}</span>
            </div>
            <span class="map" style=format!("grid-row: {};", idx + 1)>{demo.map}</span>
            <a class="entry" href=format!("demo/{}/scoreboard", demo.id) style=format!("grid-row: {};", idx + 1)></a>
    }
}
