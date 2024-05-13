use web_sys::window;
use cult_common::{DtoJeopardyBoard, Vector2D, WebsocketSessionEvent};
use yew::prelude::*;
use crate::game::boardbutton::BoardButton;
use crate::types::WebsocketCallback;


#[derive(Properties, PartialEq, Debug)]
pub struct BoardProps {
    pub board: DtoJeopardyBoard,
    pub onclick: WebsocketCallback,
}

pub(crate) struct Board {}

impl Component for Board {
    type Message = ();
    type Properties = BoardProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let num_categories = ctx.props().board.categories.len();
        let grid_columns = format!("repeat({}, 1fr)", num_categories);

        html! {
            <main class="jeopardy-container">
                <div class="jeopardy-board" style={format!("grid-template-columns: {}", grid_columns)}>
                    {
                        ctx.props().board.categories.iter().enumerate().map(|(row_index, category)| {
                            html! {
                                <div class="jeopardy-category">
                                    <h2>{&category.title}</h2>
                                    {
                                        category.questions.iter().enumerate().map(|(col_index, question)| {
                                            let vec_2d = Vector2D { x: row_index, y: col_index };
                                            html! {
                                                <div class="jeopardy-question">
                                                    <BoardButton dtoq={question.clone()} onclick={ctx.props().onclick.clone()} {vec_2d}/>
                                                </div>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </main>
        }
    }
}
