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
        let test1 = "100€";
        let test2 = "200€";
        let test3 = "300€";
        let cultpardy = "cult-pardy";
        let header1 = "🌟 Anime";
        let header2 = "🎨 Art";
        let header3 = "🗻 Japan";
        let header4 = "🎹 Music";
        let header5 = "🍿 Movies and watchables";
        html! {
        <main>
            <div class="listcontainer">
                <ul>
                  <li>
                      <ul>
                        <h2>{header1}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header2}</h2>
                        <div class="button-container"><button>{test1} </button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header3}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header4}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header5}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                </ul>
            </div>
        </main>
                }
    }
}
