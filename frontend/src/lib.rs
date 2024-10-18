use leptos::*;

pub mod demo;
pub use demo::Demo;

mod navbar;
pub use navbar::TopBar;

pub mod homepage;
pub use homepage::Homepage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoUploadStatus {
    Hidden,
    Shown,
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
