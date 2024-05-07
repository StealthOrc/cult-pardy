use cult_common::{DtoQuestion, Vector2D};
use yew::prelude::*;

use crate::types::AppMsg;

#[derive(Properties, Clone, PartialEq, Default)]
pub struct ButtonProps {
    #[prop_or_default]
    pub dtoq: DtoQuestion,
    pub vec_2d: Vector2D,
    pub onclick: Callback<Vector2D>,
}

#[function_component]
pub fn BoardButton(props: &ButtonProps) -> Html {
    let var = props.vec_2d;
    let onclick = props.onclick.reform(move |_| var);
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
