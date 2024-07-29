
use cult_common::wasm_lib::{ids::usersession::UserSessionId, DiscordUser};
use yew::prelude::*;


#[derive(Properties, PartialEq)]
pub struct PlayerPanelProperties {
    pub user_session_id: UserSessionId,
    pub discord_user: Option<DiscordUser>,
}
#[derive(Debug)]
pub struct PlayerProfile {}

impl Component for PlayerProfile {
    type Message = ();

    type Properties = PlayerPanelProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        PlayerProfile{}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let player = ctx.props().discord_user.clone();
        let username: String;
        let avatar_url: String;
        match player {
            Some(discord_user) => {
                username = discord_user.username.clone();
                avatar_url = discord_user.avatar_image_url();
            }
            None => {
                username = ctx.props().user_session_id.id.clone();
                avatar_url = String::from(
                    "https://upload.wikimedia.org/wikipedia/en/7/71/Franxx_Zero_Two.jpg",
                );
            }
        };
        html! {
            <div class={classes!("profile-panel")}>
                <p>{username}</p>
                <img src={avatar_url}/>
            </div>
        }
    }
}
