use cult_common::DtoJeopardyBoard;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct QuestionProps {
    pub board: DtoJeopardyBoard,
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
        html! {
            <div class={classes!("question-container")}></div>
        }
    }
}
