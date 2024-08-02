<script lang="ts" type="module">
	import { JeopardyBoardStore } from "$lib/stores/JeopardyBoardStore";
	import { SessionPingsStore } from "$lib/stores/SessionPings";
	import { WebsocketStore } from "$lib/stores/WebsocketStore";
	import type { DtoQuestion, DTOSession, Vector2D, WebsocketSessionEvent } from "cult-common";
	import { WebSocketSubject } from "rxjs/webSocket";


    export let session : DTOSession
    let default_avatar : string= "https://cdn-icons-png.flaticon.com/512/149/149071.png"

    let current : DtoQuestion | null = null;
    JeopardyBoardStore.subscribe(value => {
        if (value != null) {
            current = value.current;
        }
    })

    let ping: number = 0; 
    SessionPingsStore.get_ping_update_by_session_id(session.user_session_id,value => {
        ping = value.ping;
    })


    let ws : WebSocketSubject<WebsocketSessionEvent>;
    if (WebsocketStore != null) {
        WebsocketStore.subscribe(value => {
        ws = value;
        })
    }
    
    function getAvatar() {
        if (session.discord_user === null) {
            return default_avatar
        }
        const discord_user = session.discord_user;
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
        let store: WebsocketSessionEvent = {AddUserSessionScore : [session.user_session_id, current.vector2d]};
        ws.next(store);
    }

    function get_ping_class() {
        if (ping <= 50) {
            return 'text-green-600'
        } else if (ping <= 100) {
            return 'text-yellow-600'
        } else {
            return 'text-red-600'
        }
    }

 



</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class={`player-card hover:border-blue-500 border border-neutral-300 border-2 flex items-center border-rounded rounded radius-10 p-2 m-2 gap-2 w-full max-w-48 overflow-hidden box-border bg-neutral-200 shadow hover:shadow-lg hover:-translate-y-2 duration-200 relative ${$$props.class || ''}`} on:click={addStore}>
    {#key session.score}
        <img src="{getAvatar()}" alt="Avatar" class="h-14 w-14 rounded-full">
        <div class="flex flex-col w-full overflow-hidden">
            <p class="text-base font-bold overflow-hidden text-ellipsis">{getUserName(session)}</p> 
            <p class="m-0 text-lg text-gray-500">{session.score}</p>
        </div>
        <div class="absolute bottom-0 right-0 mx-1 font-bold {get_ping_class()}">
            <div class="flex">
            {#if ping > 999} 
                :c
            {:else if ping < 1}
               ...
            {:else}
                {ping}ms
            {/if}
            </div>  
        </div>
    {/key}
</div>

<style>

</style>