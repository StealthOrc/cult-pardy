mod app;
mod service;
mod file;
mod game;
mod ws;
mod mainpage;

use yew::prelude::*;
use yew_router::prelude::*;
//testing purposes
use gloo_storage::Storage;
use crate::app::App;
use crate::mainpage::MainPage;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
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
        Route::Home => html! {  <MainPage /> },
        Route::Game => html!{ <App  />} ,
        _ => html! { <h1>{ "404" }</h1> },
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
