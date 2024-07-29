use cult_common::dto::DTOSession;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use ritelinked::LinkedHashMap;
use yew::prelude::*;
use crate::game::playerpanel::PlayerPanel;

use crate::types::OptionalWebsocketCallback;

#[derive(Properties, PartialEq)]
pub struct PlayerListPanelProperties {
    pub creator:UserSessionId,
    pub user_list: LinkedHashMap<UserSessionId, DTOSession>,
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
        let mut html = Vec::<Html>::new();
        for session in ctx.props().user_list.values() {
            let add_user_score = ctx.props().add_user_score.clone();
            let creator = ctx.props().creator.eq(&session.user_session_id);
            html.push(
                html! {
                    <PlayerPanel creator={creator} player={session.clone()} add_user_score={add_user_score} />});
        }

        html!
        {
        <div>
            { for html.into_iter() }
        </div>
    }
    }
    // add code here
}
