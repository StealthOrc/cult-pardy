use yew::prelude::*;

mod app;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
