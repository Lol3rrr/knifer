use leptos::*;
use leptos_router::*;

use frontend::{UploadDemo, TopBar, Homepage, Demo};

async fn load_demos() -> Vec<common::BaseDemoInfo> {
    let res = reqwasm::http::Request::get("/api/demos/list").send().await.unwrap();
    let demos: Vec<common::BaseDemoInfo> = res.json().await.unwrap();

    demos
}

fn main() {
    let async_data = create_resource(|| (), |_| async move {
        load_demos().await
    });

    let (upload_demo_read, upload_demo_write) = create_signal(frontend::DemoUploadStatus::Hidden);

    mount_to_body(move || view! {
        <Router>
            <nav>
                <TopBar update_demo_visible=upload_demo_write />
            </nav>
            <main>
                <UploadDemo shown=upload_demo_read update_shown=upload_demo_write />

                <Routes>
                    <Route path="/" view=Homepage />
                    <Route path="/demo/:id" view=Demo>
                        <Route path="perround" view=frontend::demo::PerRound />
                        <Route path="" view=frontend::demo::Scoreboard />
                    </Route>
                </Routes>
            </main>
        </Router>
    })
}
