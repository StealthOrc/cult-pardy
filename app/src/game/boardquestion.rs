use cult_common::{DtoJeopardyBoard, DtoQuestion, WebsocketSessionEvent};
use gloo_console::log;
use wasm_bindgen::prelude::*;
use web_sys::console::warn_0;
use yew::prelude::*;

use crate::{types::WebsocketCallback, wasm_lib};

const iframename: &str = "existing-iframe-example";

pub enum QuestionMsg {
    YTPlayerNotYetSetup,
}

#[derive(Properties, PartialEq)]
pub struct QuestionProps {
    pub question: DtoQuestion,
    pub onclick: WebsocketCallback,
}

#[derive(Debug)]
pub(crate) struct BoardQuestion {}

impl Component for BoardQuestion {
    type Message = QuestionMsg;

    type Properties = QuestionProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let question_text = props
            .question
            .question_text
            .clone()
            .expect("did not find question_text!");

        let onclick = props.onclick.reform(move |_| WebsocketSessionEvent::Back);
        //Youtube: see https://developers.google.com/youtube/player_parameters for reference
        //TODO: add origin as youtube parameter
        ctx.link().send_message(QuestionMsg::YTPlayerNotYetSetup);

        html! {
        <div id={"question-container"}>
                <button onclick={onclick.clone()}>{"Return to board"}</button>
                <div class="yt-player-container">
                    <div id="player"/>
                </div>
                <button>{"Start Video"}</button>
                <button>{"Stop Video"}</button>
        </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            QuestionMsg::YTPlayerNotYetSetup => {
                wasm_lib::createYTPlayer("mLW35YMzELE".to_string());
                false
            }
            _ => true,
        }
    }
}
