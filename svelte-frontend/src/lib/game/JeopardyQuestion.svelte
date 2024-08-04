<script lang="ts">
	import { WebsocketStore } from '$lib/stores/WebsocketStore';
	import type { DtoQuestion, Vector2D, VideoEvent, WebsocketSessionEvent } from 'cult-common';
	import type { WebSocketSubject } from 'rxjs/webSocket';
	import { onMount } from 'svelte';
	import { on } from 'svelte/events';
	import { match, P } from 'ts-pattern';
    import YouTubePlayerPlus from 'youtube-player-plus';
	import type { YTPP_Options } from 'youtube-player-plus/types';
	import JeopardyBoard from './JeopardyBoard.svelte';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
    
    export let question: DtoQuestion;
    let open_request = false;

    let ws : WebSocketSubject<WebsocketSessionEvent> | null = null;
    if (WebsocketStore != null) {
        WebsocketStore.subscribe(value => {
            ws = value;
        })
    }

    let current : DtoQuestion | null = null;
    JeopardyBoardStore.subscribe(value => {
        if (value != null) {
            current = value.current;
        }
    })


    function handleClose() {
        if (ws == null) {
            return;
        }
        let click : WebsocketSessionEvent = "Back";
        ws.next(click);
    }
    function handleKeyDown(event: KeyboardEvent) {
        console.log(event);
        if (event.key === 'Escape') {
            handleClose();
        }
    }
    function req_open_question() {
        if (open_request || ws == null ) {
            return;
        }
        let click : WebsocketSessionEvent = {Click : question.vector2d};
        ws.next(click);
    }

    let player: YouTubePlayerPlus | null = null;

    function is_youtuve() : boolean {
        if (current == null) {
            return false;
        }
        let result = false;
        match(current.question_type)
        .with({ Media: P.select() }, (data) => {
            console.log(current);
            result = true;
        })
        .otherwise(() => {
            result = false;
        });
        return result;
    }

    function createYouTubePlayer() : boolean {
        if (current == null) {
            return false;
        }
        if (player != null) {
            return true;
        }
        let result = false;
        match(current.question_type)
        .with({ Media: P.select() }, (data) => {
            //if element #player is not found, return false
            console.log("?", document.getElementsByClassName("player").length > 0);
            if (document.getElementsByClassName("player").length == 0) {
                result = false;
                return
            }

            let options : YTPP_Options = {
                autoplay: true,
                controls: false,
                keyboard: false,
                loop: false,
                annotations: false,
                modestBranding: false,
                relatedVideos: false,
                playsInline: false,
            }

            player = new YouTubePlayerPlus('#player', options)
            player.load(data)
            player.setVolume(100)
            result = true;
        })
        .otherwise(() => {
            result = false;
        });
        return result;
    }

    function play() {
        if (player == null) {
            return;
        }
        if (open_request || ws == null ) {
            return;
        }
        player.play();
        let click : VideoEvent  = "Play" ;
        let click2 : WebsocketSessionEvent = {ViedeoEvent : click};
        ws.next(click2);
    }

    function pause() {
        if (player == null) {
            return;
        }
        if (open_request || ws == null ) {
            return;
        }
        player.pause();
        let click : VideoEvent ={ Pause: 12 };
        let click2 : WebsocketSessionEvent = {ViedeoEvent : click};
        ws.next(click2);
    }

    function stop() {
        if (player == null) {
            return;
        }
        if (open_request || ws == null ) {
            return;
        }
        player.stop();
        let click : VideoEvent ={ Resume: 12 };
        let click2 : WebsocketSessionEvent = {ViedeoEvent : click};
        ws.next(click2);
    }


</script>
<div class="player" id="player"></div>
<div class="jeopardy-question">
    {#if question.won_user_id !== null}
        <button disabled>WON</button>
    {:else}
        <button on:click={req_open_question}>${question.value}</button>
    {/if}
    {#if current && current.vector2d.x === question.vector2d.x && current.vector2d.y === question.vector2d.y}
        <div class="overlay" role="dialog">
            {#if is_youtuve()}
                <div class="overlay-content">
                    //edit hier with tailwind
                    <p class="player container mx-auto"></p>
                    <h1>${current.value}</h1>
                    <p>{current.question_type}</p>
                    {#if createYouTubePlayer()}
                     
                    <button on:click={() => play()}>Play</button>
                    <button on:click={() => pause()}>Pause</button>
                    <button on:click={() => stop()}>Stop</button>
                {/if}
                </div>
            {:else}
                <div class="overlay-content">
                    <h1>${current.value}</h1>
                    <p>{current.question_text}</p>
                </div>
            {/if}
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