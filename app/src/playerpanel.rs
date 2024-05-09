use cult_common::DiscordUser;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlayerPanelProperties {
    pub player: DiscordUser,
}
#[derive(Debug)]
pub struct PlayerPanel {}

impl Component for PlayerPanel {
    type Message = ();

    type Properties = PlayerPanelProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let player = ctx.props().player.clone();
        html! {
            <div class={classes!("player-panel")}>
                <p>{ctx.props().player.username.clone()}</p>
                <img src={player.avatar_image_url()}/>
            </div>
        }
    }
}
