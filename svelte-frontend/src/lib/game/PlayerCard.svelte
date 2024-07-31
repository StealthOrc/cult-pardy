<script lang="ts" type="module">
	import type { Session } from "$lib/stores/currentSessions";
	import type { DTOSession, Vector2D, WebsocketSessionEvent } from "cult-common";
	import type { WebSocketSubject } from "rxjs/webSocket";

    export let session : Session
    export let current: Vector2D | null;
    export let ws : WebSocketSubject<any> | null;
    let default_avatar : string= "https://cdn-icons-png.flaticon.com/512/149/149071.png"



    function getAvatar() {
        if (session.dto_Session.discord_user === null) {
            return default_avatar
        }
        const discord_user = session.dto_Session.discord_user;
        let avatar_id = discord_user.avatar_id
        let discord_id = discord_user.discord_id
        return `https://cdn.discordapp.com/avatars/${discord_id.id}/${avatar_id}.png?size=64*10`;
    }
    

    function getUserName(DTOSession: DTOSession) {
        if (DTOSession.discord_user === null) {
            return DTOSession.user_session_id.id
        }
        return DTOSession.discord_user.username
    }

    function addStore() {
        if (ws == null || session == null || current == null) {
            return;
        }
        let store: WebsocketSessionEvent = {AddUserSessionScore : [session.user_session_id, current]};
        ws.next(store);
    }



</script>

<div class="player-card">
    {#key session.dto_Session.score}
        <img src="{getAvatar()}" on:click={addStore} alt="Avatar" class="player-avatar">
        <h1 class="player-username">{getUserName(session.dto_Session)}</h1> 
        <h3 class="player-score">{session.dto_Session.score}</h3>
        <h2 class="ping">{session.ping}</h2>
    {/key}
</div>



<style>
    .player-card {
        background-color: #f1f1f1;
        border: 1px solid #ddd;
        border-radius: 5px;
        padding: 10px;
        margin: 10px;
        box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        max-width: 200px; /* Adjust based on your design */
        box-sizing: border-box; /* Include padding and border in the width */
        overflow: hidden; /* Prevent overflow */
    }

    .player-avatar {
        width: 50px;
        height: 50px;
        border-radius: 50%;
        flex-shrink: 0; /* Prevent avatar from shrinking */
    }

    .player-username {
        margin: 0;
        font-size: 18px;
        font-weight: bold;
        white-space: nowrap; /* Prevent text wrapping */
        overflow: hidden;
        text-overflow: ellipsis; /* Truncate text with ellipsis */
    }

    .player-score {
        margin: 0;
        font-size: 18px;
        color: #666;
    }
</style>