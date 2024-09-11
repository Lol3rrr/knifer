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
    view! {
        <li>
            <A href=format!("demo/{}", demo.id)>Demo: {demo.map} - {demo.id}</A>
        </li>
    }
}

#[leptos::component]
pub fn upload_demo(shown: ReadSignal<DemoUploadStatus>, update_shown: WriteSignal<DemoUploadStatus>) -> impl leptos::IntoView {
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
    let demo_data = create_resource(|| (), |_| async move {
        let res = reqwasm::http::Request::get("/api/demos/list").send().await.unwrap();
        let demos: Vec<common::BaseDemoInfo> = res.json().await.unwrap();
        demos
    });

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
