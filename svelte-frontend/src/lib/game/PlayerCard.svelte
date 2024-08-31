<script lang="ts" type="module">
	import { CookieStore, type SessionCookies } from "$lib/stores/cookies";
	import { JeopardyBoardStore } from "$lib/stores/JeopardyBoardStore";
	import { SessionPingsStore } from "$lib/stores/SessionPings";
	import { CurrentSessionsStore } from "$lib/stores/SessionStore";
	import { WebsocketStore } from "$lib/stores/WebsocketStore";
	import type { DtoQuestion, DTOSession, Vector2D, WebsocketSessionEvent } from "cult-common";
    
    type Props = {
        session: DTOSession,
    }

    let { session } : Props = $props();

    const default_avatar: string = "https://cdn-icons-png.flaticon.com/512/149/149071.png"

    let ping: number = $state(0);
    SessionPingsStore.subscribe(value => {
        ping = value.find(ping => ping.user_session_id.id === session.user_session_id.id)?.ping || 0;
    });
    
    let currentQuestion : DtoQuestion | undefined = $state(undefined); 
    let canAddScore: boolean = $state(false);
    let browserSessionIsAdmin = CurrentSessionsStore
            .getSessionById({ id: $CookieStore.userSessionId.id})
            .is_admin;
     
    JeopardyBoardStore?.subscribe(value => {
        if (value == null) {
            return;
        }
        currentQuestion = value.current;
        canAddScore = browserSessionIsAdmin && currentQuestion != undefined;
    })


    let ws = $WebsocketStore.webSocketSubject
    
    function getAvatar() {
        if (!session || session.discord_user === null) {
            return default_avatar
        }
        const discord_user = session.discord_user;
        if (discord_user != null) {
            let avatar_id = discord_user.avatar_id
            let discord_id = discord_user.discord_id
            return `https://cdn.discordapp.com/avatars/${discord_id.id}/${avatar_id}.png?size=64*10`;
        }
    }
    

    function getUserName(DTOSession: DTOSession) {
        if (DTOSession.discord_user != null) {
            return DTOSession.discord_user.username
        }
        return DTOSession.user_session_id.id
    }

    function addStore() {
        if (ws == null || session == null || currentQuestion == undefined) {
            return;
        }
        let store: WebsocketSessionEvent = {AddUserSessionScore : [session.user_session_id, currentQuestion.vector2d]};
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
<div onclick={canAddScore? addStore : (): void => {}} class={`flex items-center border-2 border-cultGrey border-rounded rounded radius-10 p-2 m-2 gap-2 w-full max-w-48 overflow-hidden box-border bg-cultGrey shadow hover:shadow-lg hover:-translate-y-2 duration-200 relative ${canAddScore ? 'cursor-pointer hover:border-cultPink' : 'cursor-default'}`}>
    {#key session.score}
        <img src="{getAvatar()}" alt="Avatar" class="h-14 w-14 rounded-full">
        <div class="flex flex-col w-full overflow-hidden">
            <div class="flex flex-row items-center">
                {#if session.is_admin}
                    <p class="text-red-500 text-sm font-bold mr-1">[A]</p>
                {/if}
                <p class="text-base text-white font-semibold overflow-hidden text-ellipsis">{getUserName(session)}</p> 
            </div>
            <p class="m-0 text-lg text-cultGrey-light">{session.score}</p>
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