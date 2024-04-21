use std::fmt::write;

use yew::prelude::*;

pub enum Msg {
    TestMessage(String),
}

#[derive(Debug, Default)]
pub struct App {
    test_data: String,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TestMessage(test) => self.test_data = test,
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {<main>
            <div class="grid-container">
              <div class="grid-item">{1}</div>
              <div class="grid-item">{2}</div>
              <div class="grid-item">{3}</div>
              <div class="grid-item">{4}</div>
            </div>
        </main>
        }
    }
}
