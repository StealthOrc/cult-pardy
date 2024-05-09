use gloo_console::log;
use yew::prelude::*;

use crate::playerpanel::*;
use crate::types::DiscordUserList;

#[derive(Properties, PartialEq)]
pub struct PlayerListPanelProperties {
    pub user_list: DiscordUserList,
}

pub struct PlayerListPanel {}

impl Component for PlayerListPanel {
    type Message = ();

    type Properties = PlayerListPanelProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let player_list = match ctx.props().user_list.clone() {
            Some(user_list) => user_list,
            None => vec![],
        };

        log!(format!(
            "PlayerListPanel view(): player_list: {:?}",
            player_list
        ));
        // player_list.iter().ma {
        //     html! {<PlayerPanel {player}/>};
        // }
        // let player = DiscordUser {
        //     discord_id: DiscordID {
        //         id: String::from("172348658409275393"),
        //     },
        //     username: String::from("stealthorc"),
        //     avatar_id: String::from("514fd3385605e5caa32dce4d260281c5"),
        //     discriminator: String::from("discriminator"),
        //     global_name: String::from("global_name"),
        // };
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
