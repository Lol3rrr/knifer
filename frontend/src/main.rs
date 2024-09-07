use leptos::*;
use leptos::prelude::*;

async fn load_demos() -> usize {
    let res = reqwasm::http::Request::get("/api/demos/list").send().await.unwrap();
    dbg!(res);

    0
}

fn main() {
    let async_data = create_resource(|| (), |_| async move {
        load_demos().await
    });

    mount_to_body(move || view! {
        <p>"Hello, world!"</p>
        <a href="/api/steam/login">Steam Login</a> { move || match async_data.get() {
            None => 123,
            Some(v) => v,
        } }

        <form action="/api/demos/upload" method="post" enctype="multipart/form-data">
            Select File to upload
            <input type="file" name="demo" id="demo"></input>
            <input type="submit" value="Upload Image" name="submit"></input>
        </form>
    })
}
