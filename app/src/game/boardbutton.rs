use cult_common::{dto::DtoQuestion, wasm_lib::{websocketevents::WebsocketSessionEvent, Vector2D}};
use yew::prelude::*;

use crate::types::*;

#[derive(Properties, Clone, PartialEq, Default)]
pub struct ButtonProps {
    #[prop_or_default]
    pub dtoq: DtoQuestion,
    pub vec_2d: Vector2D,
    pub onclick: WebsocketCallback,
}

#[function_component]
pub fn BoardButton(props: &ButtonProps) -> Html {
    let var = props.vec_2d;
    let onclick = props
        .onclick
        .reform(move |_| WebsocketSessionEvent::Click(var));
    if props.dtoq.won_user_id.is_some() {
        html! {
        <button onclick={onclick.clone()}>{format!("Disabled: {}€",props.dtoq.value) }</button>
        }
    } else {
        html! {
        <div class="button-container"><button onclick={onclick.clone()}>{format!("{}€",props.dtoq.value)}</button></div>
        }
    }
}
