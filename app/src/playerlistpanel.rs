use std::collections::HashMap;
use gloo_console::log;
use yew::prelude::*;
use cult_common::{DiscordUser, UserSessionId};

use crate::playerpanel::*;
use crate::types::UserList;

#[derive(Properties, PartialEq)]
pub struct PlayerListPanelProperties {
    pub user_list: UserList,
}

pub struct PlayerListPanel {}

impl Component for PlayerListPanel {
    type Message = ();

    type Properties = PlayerListPanelProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let player_list = ctx.props().user_list.values()
            .filter_map(|user| user.clone())
            .collect::<Vec<DiscordUser>>();


        log!(format!(
            "PlayerListPanel view(): player_list: {:?}",
            player_list
        ));


        html! {
            <div>
                {
                    player_list.iter().map(|player|{
                        html!{<PlayerPanel player={player.clone()}/>}
                    }).collect::<Html>()
                }
            </div>
        }
    }
    // add code here
}
