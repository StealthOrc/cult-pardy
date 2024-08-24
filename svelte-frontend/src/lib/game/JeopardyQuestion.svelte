<script lang="ts">

	import type { WebsocketSessionEvent, DtoQuestion } from 'cult-common';
	import { match, P } from 'ts-pattern';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
	import { CookieStore, type SessionCookies } from '$lib/stores/cookies';
	import { QuestionTypes, VideoPlayerType } from '$lib/types';
	import { WebsocketStore } from '$lib/stores/WebsocketStore';
	import BlobDisplay from './blobdisplay/BlobDisplay.svelte';
	import YoutubeDisplay from './youtubedisplay/YoutubeDisplay.svelte';
    export let question: DtoQuestion;

    let ws = $WebsocketStore.webSocketSubject;
    let current : DtoQuestion | undefined = undefined;
    let type : QuestionTypes = QuestionTypes.NONE

    JeopardyBoardStore.subscribe(value => {
        if (value != null) {
            current = value.current;
            if ((current != null) && (current.vector2d.x === question.vector2d.x && current.vector2d.y === question.vector2d.y)) {
                match(question.question_type)
                .with({ Video: P.select() }, async (vid) => {
                    type = QuestionTypes.BLOB;

                })
                .with({ Youtube: P.select() }, async (aud) => {
                    type = QuestionTypes.YOUTUBE;
                })
                .otherwise(() => {
                    console.log("Unsupported file type");
                });
          
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
        let click : WebsocketSessionEvent = {Click : question.vector2d};
        ws.next(click);
    }

</script>

<div class="jeopardy-question">
    {#if question.won_user_id !== null}
        <button disabled>WON</button>
    {:else}
        <button on:click={req_open_question}>${question.value}</button>
    {/if}
    {#if current && current.vector2d.x === question.vector2d.x && current.vector2d.y === question.vector2d.y}
        <div class="overlay" role="dialog">
                <div class="overlay-content">
                    {#if type == QuestionTypes.BLOB}
                        <BlobDisplay current={current}/>
                    {:else if type == QuestionTypes.YOUTUBE}
                        <YoutubeDisplay current={current}/>
                    {:else}
                        <h1>${current.value}</h1>
                        <p>{current.question_text}</p>
                    {/if}
                </div>
            <div id="ov"></div>
            <button class="close-button" on:click={handleClose}>Close</button>
    </div>
    {/if}
</div>


<style>
    .close-button {
        top: 10px;
        right: 10px;
        background-color: #f44336;
        color: white;
        border: none;
        border-radius: 5px;
        cursor: pointer;
        transition: background-color 0.3s ease;
        position: absolute;



    }
    .jeopardy-question {
        margin: 5px;
        position: relative;
    }

    .jeopardy-question button {
        width: 100px;
        height: 60px;
        font-size: 24px;
        background-color: #4CAF50;
        color: white;
        border: none;
        border-radius: 5px;
        cursor: pointer;
        transition: background-color 0.3s ease;
    }

    .jeopardy-question button:hover {
        background-color: #45a049;
    }

    .overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 2;
    }


    .overlay-content {
        background-color: white;
        padding: 20px;
        border-radius: 10px;
        box-shadow: 0 0 10px rgba(0, 0, 0, 0.3);
        max-width: 80%;
        max-height: 80%;
        overflow-y: auto;
    }
</style>