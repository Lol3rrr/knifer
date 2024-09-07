use leptos::*;

#[leptos::component]
pub fn demo() -> impl leptos::IntoView {
    let params = leptos_router::use_params_map();
    let id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    let demo_info = create_resource(|| (), move |_| async move {
        let res = reqwasm::http::Request::get(&format!("/api/demos/{}/info", id())).send().await.unwrap();
        dbg!(res.text().await);
        0
    });

    view! {
        <h2>Demo - {id}</h2>
    }
}
