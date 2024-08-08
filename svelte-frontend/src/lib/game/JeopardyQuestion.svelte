<script lang="ts">
	import { WebsocketStore } from '$lib/stores/WebsocketStore';
	import type { ApiResponse, CFile, DtoQuestion, FileChunk, Vector2D, WebsocketSessionEvent } from 'cult-common';
	import type { WebSocketSubject } from 'rxjs/webSocket';
	import { onMount } from 'svelte';
	import { on } from 'svelte/events';
	import { match, P } from 'ts-pattern';
    import YouTubePlayerPlus from 'youtube-player-plus';
	import type { YTPP_Options } from 'youtube-player-plus/types';
	import JeopardyBoard from './JeopardyBoard.svelte';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
	import { binary_test } from '$lib/const';
	import { get_file } from '$lib/api/ApiRequests';
	import { buildUint8ArrayFromChunks} from '$lib/BinaryConversion';
    
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

    function binaryToByteArray(binaryString: string): Uint8Array {
        if (binaryString.length % 8 !== 0) {
            throw new Error('Binary string length must be a multiple of 8');
        }
        
        const byteArray = new Uint8Array(binaryString.length / 8);
        
        for (let i = 0; i < byteArray.length; i++) {
            byteArray[i] = parseInt(binaryString.slice(i * 8, (i + 1) * 8), 2);
        }
        
        return byteArray;
    }
    // Function to load the video into a Blob
    let videoBlobUrl: string;
    async function loadVideoToBlob(url: string) {
        try {
            const data: CFile = await get_file('FlyHigh.mp4');

                console.log("loadVideoToBlob",);
                let dataChunks: FileChunk[] = data.file_chunks;
                dataChunks.sort((a, b) => a.index - b.index);
                let chunks: Uint8Array[] = [];
                for (let i = 0; i < data.file_chunks.length; i++) {
                    chunks.push(new Uint8Array(dataChunks[i].chunk));
                }
                const buf: ArrayBuffer = buildUint8ArrayFromChunks(chunks);

                const videoBlob: Blob = new Blob([buf], { type: 'video/mp4' });

                // Create a URL for the Blob
                videoBlobUrl = URL.createObjectURL(videoBlob);
            
            
        } catch (error) {
            console.error('Error loading video:', error);
        }
    }

    // Load the video when the component is mounted
    onMount(() => {
        //todo: DEBUG ONLY!! load video for all instances!
        if (question && question.vector2d.x == 0 && question.vector2d.y == 1) {
            const videoUrl = '/assets/FlyHigh.mp4';
            loadVideoToBlob(videoUrl);
        }
    });
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
            <div class="overlay-content">
                {#if videoBlobUrl}
                    <!-- Render the video element once the Blob URL is available -->
                    <video src={videoBlobUrl} controls>
                        <track kind="captions" />
                    </video>
                {:else}
                    <p>Loading video...</p>
                {/if}
                {#if is_youtuve()}
                    <!--edit hier with tailwind -->
                    <p class="player container mx-auto"></p>
                    <h1>${current.value}</h1>
                    <p>{current.question_type}</p>

                    {#if createYouTubePlayer()}

                    <button on:click={() => player?.play()}>Play</button>
                    <button on:click={() => player?.pause()}>Pause</button>
                    <button on:click={() => player?.stop()}>Stop</button>
                    {/if}
                {:else}
                    <h1>${current.value}</h1>
                    <p>{current.question_text}</p>
                {/if}
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