use leptos::*;
use leptos_router::A;

mod demo;
pub use demo::Demo;

#[leptos::component]
pub fn demo_list_entry(demo: common::BaseDemoInfo) -> impl leptos::IntoView {
    view! {
        <li>
            <A href=format!("/demo/{}", demo.id)>Demo: {demo.id}</A>
        </li>
    }
}

#[leptos::component]
pub fn steam_login(height: &'static str, width: &'static str) -> impl leptos::IntoView {
    let user_status = create_resource(|| (), |_| async move {
        let res = reqwasm::http::Request::get("/api/user/status").send().await.unwrap();
        res.status() == 200
    });

    let tmp = move || if user_status.get().unwrap_or(false) {
        view! {
            <p>Logged in</p>
        }.into_any()
    } else {
        view! {    
            <a href="/api/steam/login">
                <img src="https://community.akamai.steamstatic.com/public/images/signinthroughsteam/sits_01.png" alt="Steam Login" style=format!("height: {height}; width: {width}") />
            </a>
        }.into_any()
    };

    view! {
        { tmp }
    }
}

#[leptos::component]
pub fn upload_demo() -> impl leptos::IntoView {
    use leptos_router::Form;

    view! {
        <div>
            <Form action="/api/demos/upload" method="post" enctype="multipart/form-data".to_string()>
                <p> Select File to upload </p>
                <input type="file" name="demo" id="demo"></input>
                <input type="submit" value="Upload Image" name="submit"></input>
            </Form>
        </div>
    }
}

#[leptos::component]
pub fn top_bar() -> impl leptos::IntoView {
    let style = stylers::style! {
        "TopBar",
        .bar {
            width: 100%;
            height: 4vh;
            padding-top: 0.5vh;
            padding-bottom: 0.5vh;

            background-color: #28282f;
            color: #d5d5d5;

            display: grid;
            grid-template-columns: 15vw auto auto calc(4vh * (180/35) + 20px);
        }

        .group {
            display: block;
            width: 30vw;
        }

        .elem {
            display: inline-block;
        }

        .logo {
            color: #d5d5d5;
            width: 15vw;
            font-size: 24px;
            padding: 0px;
            margin: 0px;
        }
    };

    view! {class = style,
        <div class="bar">
            <A href="/">
                <p class="logo">Knifer</p>
            </A>
            
            <div class="elem">
                Upload Demo
            </div>

            <div class="elem" style="grid-column-start: 4">
                <SteamLogin height="4vh" width="auto" />
            </div>
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
                <UploadDemo />
            </div>
            <ul>
                { move || demo_data.get().unwrap_or_default().into_iter().map(|demo| crate::DemoListEntry(DemoListEntryProps {
            demo
        })).collect::<Vec<_>>() }
            </ul>
        </div>
    }
}
