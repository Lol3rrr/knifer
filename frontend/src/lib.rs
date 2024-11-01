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
            min-height: 18vh;
            padding: 1vh 1vw;

            color: #f1f1f1;
            background-color: #42424d;

            border-radius: 10px;

            display: none;
        }

        .hidden {
            display: none;
        }
        .shown {
            display: block;
        }
    };

    let uploading = RwSignal::new(false);

    let handle_resp: std::rc::Rc<dyn Fn(&leptos::web_sys::Response)> = std::rc::Rc::new(move |resp: &leptos::web_sys::Response| {
        if resp.status() != 200 {
            // TODO
            // Display error somehow
            return;
        }

        uploading.set(false);

        // Remove the Upload popup
        update_shown.set(DemoUploadStatus::Hidden);

        // TOOD
        // Reload the demo list
    });

    let on_submit: std::rc::Rc<dyn Fn(&leptos::web_sys::FormData)> = std::rc::Rc::new(move |_| {
        uploading.set(true);
    });

    view! {class = style,
        <div class="container" class:shown=move || shown() == DemoUploadStatus::Shown>
            <h3>Upload a Demo</h3>

            <Form action="/api/demos/upload" method="post" enctype="multipart/form-data".to_string() on_response=handle_resp on_form_data=on_submit>
                <p> Select Demo to upload </p>
                <input type="file" name="demo" id="demo"></input>
                <input type="submit" value="Upload Demo" name="submit"></input>
            </Form>
            <button on:click=move |_| update_shown(DemoUploadStatus::Hidden)>
                Close
            </button>

            <p 
                class:shown=move || uploading.get()
                class:hidden=move || !uploading.get()
            >Uploading...</p>
        </div>
    }
}
