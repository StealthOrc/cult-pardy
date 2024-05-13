use cult_common::{DtoJeopardyBoard, DtoQuestion, WebsocketSessionEvent};
use yew::prelude::*;

use crate::types::WebsocketCallback;

#[derive(Properties, PartialEq)]
pub struct QuestionProps {
    pub question: DtoQuestion,
    pub onclick: WebsocketCallback,
}

#[derive(Debug)]
pub(crate) struct BoardQuestion {}

impl Component for BoardQuestion {
    type Message = ();

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

        let onclick = props
            .onclick
            .reform(move |_| WebsocketSessionEvent::Back);
        return html! {
            <button onclick={onclick.clone()}>{format!("Disabled: {}€",question_text) }</button>
        }
    }
}
