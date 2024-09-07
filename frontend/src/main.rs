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

    mount_to_body(move || view! {
        <Router>
            <nav>
                <TopBar />
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=Homepage />
                    <Route path="/demo/:id" view=Demo />
                </Routes>
            </main>
        </Router>
    })
}
