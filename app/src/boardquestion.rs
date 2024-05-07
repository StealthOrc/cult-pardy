use cult_common::{DtoJeopardyBoard, DtoQuestion};
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
        html! {
            <div class={classes!("question-container")}>{question_text}</div>
        }
    }
}
