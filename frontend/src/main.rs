use leptos::*;
use leptos_router::*;

use frontend::{Demo, Homepage, TopBar, UploadDemo};

fn main() {
    let (upload_demo_read, upload_demo_write) = create_signal(frontend::DemoUploadStatus::Hidden);

    mount_to_body(move || {
        view! {
            <Router>
                <nav>
                    <TopBar update_demo_visible=upload_demo_write />
                </nav>
                <main>
                    <UploadDemo shown=upload_demo_read update_shown=upload_demo_write />

                    <Routes>
                        <Route path="/" view=Homepage />
                        <Route path="/demo/:id" view=Demo>
                            <Route path="perround" view=frontend::demo::perround::PerRound />
                            <Route path="scoreboard" view=frontend::demo::scoreboard::Scoreboard />
                            <Route path="heatmaps" view=frontend::demo::heatmap::Heatmaps />
                            <Route path="" view=frontend::demo::scoreboard::Scoreboard />
                        </Route>
                    </Routes>
                </main>
            </Router>
        }
    })
}
