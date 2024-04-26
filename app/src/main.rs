mod app;
mod websocket;

use app::App;
use yew::prelude::*;
use yew_router::prelude::*;
//testing purposes
use gloo_console::log;
use gloo_storage::Storage;
use wasm_bindgen::JsValue;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[cfg(not(debug_assertions))]
    #[at("/game/:id")]
    Game,
    // in debug mode, for testing with trunk serve
    #[cfg(debug_assertions)]
    #[at("/assets/game/:id")]
    Game,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <h1> { "Home" } </h1> },
        Route::Game => html! { <App/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Main)]
fn main_comp() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
