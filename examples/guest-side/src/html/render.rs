#![allow(unused_imports)]
#![allow(non_snake_case)]

use anyhow::Result;

use yew::prelude::*;
use yew::{function_component, ServerRenderer};

use super::router::{switch, Route};

#[function_component]
fn Content() -> Html {
    use yew_router::prelude::*;

    html! {
        <>
            <h1>{"Yew WASI SSR demo"}</h1>
            <Switch<Route> render={switch} />
        </>
    }
}

#[function_component]
fn App() -> Html {
    use yew_router::{
        history::{AnyHistory, History, MemoryHistory},
        prelude::*,
    };

    let history = AnyHistory::from(MemoryHistory::new());
    history.push("/");

    html! {
        <div>
            <Router history={history}>
                <Content />
            </Router>
        </div>
    }
}

pub async fn render() -> Result<String> {
    let renderer = ServerRenderer::<App>::new();
    let html_raw = renderer.render_async().await;

    let mut body = String::new();
    body.push_str("<body>");
    body.push_str("<div id='app' style='width: 100vw; height: 100vh; position: fixed;'>");
    body.push_str(&html_raw);
    body.push_str("</div>");
    body.push_str("</body>");

    Ok(body)
}
