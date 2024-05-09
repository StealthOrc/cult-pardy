use yew::prelude::*;

use crate::playerpanel::*;
use crate::types::{OptionalWebsocketCallback, UserList};

#[derive(Properties, PartialEq)]
pub struct PlayerListPanelProperties {
    pub user_list: UserList,
    #[prop_or(None)]
    pub add_user_score: OptionalWebsocketCallback,
}

pub struct PlayerListPanel {}

impl Component for PlayerListPanel {
    type Message = ();

    type Properties = PlayerListPanelProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                {
                    ctx.props().user_list.values().map(|player|{
                        let add_user_score = ctx.props().add_user_score.clone();
                        html!{<PlayerPanel player={player.clone()} {add_user_score}/>}
                    }).collect::<Html>()
                }
            </div>
        }
    }
    // add code here
}
