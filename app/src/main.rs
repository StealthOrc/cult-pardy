

mod app;
mod websocket;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
