<script lang="ts">
	import type { DtoQuestion, Vector2D, WebsocketSessionEvent } from 'cult-common';
	import type { WebSocketSubject } from 'rxjs/webSocket';


    
    export let question: DtoQuestion;
    export let ws: WebSocketSubject<any> | null;
    export let currentQuestion : Vector2D | null;
    let open_request = false;

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
        if (open_request) {
            return;
        }
        if (ws == null) {
            return;
        }
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
    {#if currentQuestion && currentQuestion.x === question.vector2d.x && currentQuestion.y === question.vector2d.y}
        <div class="overlay" role="dialog">
            <div class="overlay-content">
                <h1>${question.value}</h1>
                <p>{question.question_text} ?</p>
                <p>Vector2D X{question.vector2d.x}</p>
                <p>Vector2D Y{question.vector2d.y}</p>
            </div>
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