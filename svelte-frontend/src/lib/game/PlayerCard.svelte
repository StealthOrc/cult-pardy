<script lang="ts" type="module">
	import type { Session } from "$lib/stores/currentSessions";
	import type { DTOSession, Vector2D, WebsocketSessionEvent } from "cult-common";
	import type { WebSocketSubject } from "rxjs/webSocket";

    export let session : Session
    export let current: Vector2D | null;
    export let ws : WebSocketSubject<any> | null;
    let default_avatar : string= "https://cdn-icons-png.flaticon.com/512/149/149071.png"

    let ping = 0;

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

<div class={`player-card hover:border-blue-500 border border-white border-2 flex items-center border-rounded rounded radius-10 p-2 m-2 gap-2 w-full max-w-48 overflow-hidden box-border bg-white shadow hover:shadow-lg hover:-translate-y-2 duration-200 relative ${$$props.class || ''}`} on:click={addStore}>
    {#key session.dto_Session.score}
        <img src="{getAvatar()}" alt="Avatar" class="h-14 w-14 rounded-full">
        <div class="flex flex-col w-full overflow-hidden">
            <p class="text-base font-bold overflow-hidden text-ellipsis">{getUserName(session.dto_Session)}</p> 
            <p class="m-0 text-lg text-gray-500">{session.dto_Session.score}</p>
        </div>
        <p class="absolute bottom-0 right-0 mx-1 font-bold {session.ping <= 50 ? 'text-green-600' : session.ping <= 100 ? 'text-yellow-600' : 'text-red-600'}">{#if session.ping > 999} :c {:else if session.ping < 1}Pinging...{:else}{session.ping}ms{/if}</p>


    {/key}
</div>

<style>
</style>