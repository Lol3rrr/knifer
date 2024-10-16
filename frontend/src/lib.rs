use leptos::*;
use leptos_router::A;

pub mod demo;
pub use demo::Demo;

mod navbar;
pub use navbar::TopBar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoUploadStatus {
    Hidden,
    Shown,
}

#[leptos::component]
pub fn demo_list_entry(demo: common::BaseDemoInfo) -> impl leptos::IntoView {
    let style = stylers::style! {
        "DemoListEntry",
        .entry {
            display: inline-grid;

            grid-template-columns: auto auto auto;

            border: solid #030303aa 1px;
            margin-top: 5px;
            margin-bottom: 5px;
        }

        .score, .map {
            padding-left: 5px;
            padding-right: 5px;

            margin-top: auto;
            margin-bottom: auto;
        }

        .date {
            display: inline-grid;

            grid-template-columns: auto;
            grid-template-rows: auto auto;
        }
    };

    view! {
        class=style,
        <li>
            <A href=format!("demo/{}/scoreboard", demo.id)>
                <div class="entry">
                    <span class="score">{demo.team2_score}:{demo.team3_score}</span>
                    <div class="date">
                        <span>{demo.uploaded_at.format("%Y-%m-%d").to_string()}</span>
                        <span>{demo.uploaded_at.format("%H-%M-%S").to_string()}</span>
                    </div>
                    <span class="map">{demo.map}</span>
                </div>
            </A>
        </li>
    }
}

#[leptos::component]
pub fn upload_demo(
    shown: ReadSignal<DemoUploadStatus>,
    update_shown: WriteSignal<DemoUploadStatus>,
) -> impl leptos::IntoView {
    use leptos_router::Form;

    let style = stylers::style! {
        "UploadDemo",
        .container {
            position: absolute;
            left: 25vw;
            top: 15vh;
            width: 48vw;
            height: 18vh;
            padding: 1vh 1vw;

            color: #f1f1f1;
            background-color: #42424d;

            border-radius: 10px;

            display: none;
        }

        .container.shown {
            display: block;
        }
    };

    view! {class = style,
        <div class="container" class:shown=move || shown() == DemoUploadStatus::Shown>
            <h3>Upload a Demo</h3>

            <Form action="/api/demos/upload" method="post" enctype="multipart/form-data".to_string()>
                <p> Select File to upload </p>
                <input type="file" name="demo" id="demo"></input>
                <input type="submit" value="Upload Image" name="submit"></input>
            </Form>
            <button on:click=move |_| update_shown(DemoUploadStatus::Hidden)>
                Close
            </button>
        </div>
    }
}

#[leptos::component]
pub fn homepage() -> impl leptos::IntoView {
    let demo_data = create_resource(
        || (),
        |_| async move {
            let res = reqwasm::http::Request::get("/api/demos/list")
                .send()
                .await
                .unwrap();
            let demos: Vec<common::BaseDemoInfo> = res.json().await.unwrap();
            demos
        },
    );

    view! {
        <div>
            <div>
                <h2>Demos</h2>
            </div>
            <ul>
                { move || demo_data.get().unwrap_or_default().into_iter().map(|demo| crate::DemoListEntry(DemoListEntryProps {
            demo
        })).collect::<Vec<_>>() }
            </ul>
        </div>
    }
}
