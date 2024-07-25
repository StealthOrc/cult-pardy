use cult_common::{DTOSession, WebsocketSessionEvent};
use cult_common::{DiscordUser, UserSessionId};
use gloo_console::log;
use yew::prelude::*;

use crate::types::{OptionalWebsocketCallback, WebsocketCallback};

#[derive(Properties, PartialEq)]
pub struct PlayerPanelProperties {
    pub player: DTOSession,
    #[prop_or(None)]
    pub add_user_score: OptionalWebsocketCallback,
    #[prop_or(false)]
    pub creator: bool,
}
#[derive(Debug)]
pub struct PlayerPanel {
    is_locked_in: bool,
}

impl Component for PlayerPanel {
    type Message = ();

    type Properties = PlayerPanelProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { is_locked_in: true }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let player = ctx.props().player.clone();
        let creator = ctx.props().creator.clone();
        let session_id = player.user_session_id.clone();
        let onclick = match ctx.props().add_user_score.clone() {
            Some(callback) => callback.reform(move |_| {
                log!("add_user_score: add score");
                WebsocketSessionEvent::AddUserSessionScore(session_id.clone())
            }),
            None => Callback::from(|_| {
                log!("add_user_score: do nothing");
            }),
        };
        let mut username: String;
        let mut avatar_url: String;
        match player.discord_user {
            Some(discord_user) => {
                username = discord_user.username.clone();
                avatar_url = discord_user.avatar_image_url();
            }
            None => {
                username = player.user_session_id.id;
                avatar_url = String::from(
                    "https://upload.wikimedia.org/wikipedia/en/7/71/Franxx_Zero_Two.jpg",
                );
            }
        };
        if player.is_admin {
            username = username + " [ADMIN] "
        }

        if creator {
            username = username + " [CREATOR] "
        }

        html! {
            <div class={(self.is_locked_in).then(||classes!("player-panel-locked-in")).unwrap_or(classes!("player-panel"))}>
                <p>{username}</p>
                <p>{format!("Score:{}",player.score)}</p>
                    <img src={avatar_url} {onclick}/>
            </div>
        }
    }
}
