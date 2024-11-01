use leptos::*;
use leptos_router::*;

use frontend::{Demo, Homepage, TopBar, UploadDemo};

fn main() {
    let (upload_demo_read, upload_demo_write) = create_signal(frontend::DemoUploadStatus::Hidden);

    let (get_reload_demos, reload_demos) = create_signal(0u8);

    mount_to_body(move || {
        view! {
            <Router>
                <nav>
                    <TopBar update_demo_visible=upload_demo_write />
                </nav>
                <main>
                    <UploadDemo shown=upload_demo_read update_shown=upload_demo_write reload_demos />

                    <Routes>
                        <Route path="/" view=move || view! { <Homepage get_notification=get_reload_demos /> } />
                        <Route path="/demo/:id" view=Demo>
                            <Route path="scoreboard" view=frontend::demo::scoreboard::Scoreboard>
                                <Route path="general" view=frontend::demo::scoreboard::general::General />
                                <Route path="utility" view=frontend::demo::scoreboard::utility::Utility />                    
                                <Route path="" view=frontend::demo::scoreboard::general::General />
                            </Route>
                            <Route path="perround" view=frontend::demo::perround::PerRound />
                            <Route path="heatmaps" view=frontend::demo::heatmap::Heatmaps />
                            <Route path="" view=frontend::demo::scoreboard::Scoreboard />
                        </Route>
                    </Routes>
                </main>
            </Router>
        }
    })
}
