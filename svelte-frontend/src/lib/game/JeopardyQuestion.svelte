<script lang="ts">
	import type { WebsocketSessionEvent, DtoQuestion, Media } from 'cult-common';
	import { match, P } from 'ts-pattern';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
	import { CookieStore } from '$lib/stores/cookies';
	import { QuestionTypes} from '$lib/types';
	import BlobDisplay from './blobdisplay/BlobDisplay.svelte';
	import YoutubeDisplay from './youtubedisplay/YoutubeDisplay.svelte';
	import BtnBack from '$lib/ui/BtnBack.svelte';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { WebsocketStore } from '$lib/stores/WebsocketStore';
    export let question: DtoQuestion;

    let ws = $WebsocketStore.webSocketSubject;
    let current : DtoQuestion | undefined = undefined;
    let type : QuestionTypes = QuestionTypes.NONE
    let media : Media | undefined = undefined;
    let youtube_id : string;

    JeopardyBoardStore.subscribe(value => {
        if (value != null) {
            current = value.current;
            if ((current != null) && (current.vector2d.x === question.vector2d.x && current.vector2d.y === question.vector2d.y)) {

                current.question_type

                match(current.question_type)
                .with({Media: P.select()}, (medias) => {
            
                    let action = $JeopardyBoardStore?.action_state
                    if (action == null) {
                        return;
                    }
                    if (typeof action == "object" && "MediaPlayer" in action) {
                        type = QuestionTypes.MEDIA;
                        console.log("MEIDA", action.MediaPlayer.current_media)
                        let id : number = action.MediaPlayer.current_media;
                        media =  medias[id];
                    }
                })
                .with({Youtube: P.select()}, (youtube) => {
                    type = QuestionTypes.YOUTUBE;
                    youtube_id = youtube;
                })
                .with("Question", () => {
                    type = QuestionTypes.QUESTION;
                })


            }
        }
    })

    function handleClose() {
        if (ws == null)  return;
        let click : WebsocketSessionEvent = "Back";
        ws.next(click);
    }
    
    function req_open_question() {
        if (ws == null ) return;
        let click : WebsocketSessionEvent = {ChooseQuestion : question.vector2d};
        ws.next(click);
    }

    function isAdmin(): boolean {
        return $CurrentSessionsStore.filter(s => s.user_session_id.id === $CookieStore.userSessionId.id && s.is_admin).length > 0;
    }
</script>

{#if current && current.vector2d.x === question.vector2d.x && current.vector2d.y === question.vector2d.y}
    <div class="cult-bg-gradient fixed flex justify-center items-center top-0 left-0 w-full h-full z-10" role="dialog">
        <div class="cult-surface p-4 w-3/4 h-2/4 overflow-y-auto text-white">
            {#if type == QuestionTypes.MEDIA && media != undefined}
                <BlobDisplay media={media}/>
            {:else if type == QuestionTypes.YOUTUBE}
                <YoutubeDisplay current={current} youtube_id={youtube_id}/>
            {:else if type == QuestionTypes.QUESTION}
                <h1>${current.value}</h1>
                <p>{current.question_text}</p>
            {:else}
                <h1>ERROR</h1>
            {/if}
        </div>
        <div id="ov"></div>
        {#if isAdmin()}
            <BtnBack onclick={handleClose} text="Close"/>
        {/if}
    </div>
{:else}
    <div class="m-1.5">
        {#if question.won_user_id !== null}
            <button class="w-24 h-14 text-2xl text-cultTurq font-semibold bg-cultGrey rounded-md shadow-md shadow-black/60 cursor-not-allowed transition-colors duration-200 ease-in-out" disabled>WON</button>
        {:else}
            <button on:click={req_open_question} class="w-24 h-14 text-2xl text-black font-semibold bg-cultTurq hover:bg-cultPink rounded-md shadow-md shadow-black/60 cursor-pointer transition-colors duration-200 ease-in-out">{question.value}</button>
        {/if}
    </div>
{/if}