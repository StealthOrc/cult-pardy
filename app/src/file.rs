
use yew::{html, Component, Html, Context};
use cult_common::JeopardyBoard;
use cult_common::JeopardyMode::NORMAL;

pub struct Model {
    json_data: String
}

pub enum Msg {
    JsonLoaded(String),
    FetchError,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: &yew::Context<Model>) -> Self {

        let json_data =JeopardyBoard::default(NORMAL);
        let json_string = serde_json::to_string(&json_data).unwrap_or_else(|_| "".to_string());
        Self { json_data: json_string }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { self.view_json_data() }
            </div>
        }
    }
}

impl Model {
    fn view_json_data(&self) -> Html {
        let data = &self.json_data;
        html! { <div>{ generate_data(data.clone()) }</div> }
    }
}


fn generate_data(s: String) -> Vec<u8> {
    // Convert the string `s` to bytes
    let s_bytes = s.into_bytes();

    // Create a vector to hold the data
    let mut data = Vec::new();

    // Extend the data vector with the byte string "b"
    data.extend_from_slice(b"b");

    // Extend the data vector with the bytes from string `s`
    data.extend(s_bytes);

    data
}