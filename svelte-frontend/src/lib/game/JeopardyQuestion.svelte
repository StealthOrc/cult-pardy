<script lang="ts">
	import { WebsocketStore } from '$lib/stores/WebsocketStore';
	import type { DtoQuestion, DTOSession, WebsocketSessionEvent } from 'cult-common';
	import type { WebSocketSubject } from 'rxjs/webSocket';
	import { match, P } from 'ts-pattern';
    import YouTubePlayerPlus from 'youtube-player-plus';
	import type { YTPP_Options } from 'youtube-player-plus/types';
	import { JeopardyBoardStore } from '$lib/stores/JeopardyBoardStore';
	import { BlobType, downloadBlob, getBlobType, type FileDownloadProgress } from './blobdisplay/blodUtils';
	import ImageBlob from './blobdisplay/ImageBlob.svelte';
	import AudioBlob from './blobdisplay/AudioBlob.svelte';
	import TextBlob from './blobdisplay/TextBlob.svelte';
	import VideoBlob from './blobdisplay/VideoBlob.svelte';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { CookieStore, type SessionCookies } from '$lib/stores/cookies';
	import { VideoPlayerType } from '$lib/types';
    
    export let question: DtoQuestion;

    let session: DTOSession;
    let cookies: SessionCookies;
    CookieStore.subscribe(value => {
       cookies = value; 
    })
    let open_request = false;
    let ws : WebSocketSubject<WebsocketSessionEvent> | null = null;
    if (WebsocketStore != null) {
        WebsocketStore.subscribe(value => {
            ws = value;
        })
    }
    let download_request = false;

    let blob: Blob | null = null;
    let current : DtoQuestion | null = null;

    let file_download_progress: FileDownloadProgress | null = null;
    let videoType: VideoPlayerType = VideoPlayerType.NONE;

    function isAdmin(): boolean {
        return CurrentSessionsStore
            .getSessionById({ id: cookies.userSessionId.id })
            .is_admin;
    }

    const onProgress = (progress: FileDownloadProgress) => {
        console.log('Progress:', progress);
        if (progress.blob != null) {
            blob = progress.blob;
        } else {
            file_download_progress = progress;
        }
    };

    async function loadVideoToBlob(video: string) {
        try {
            if (!download_request && blob == null) {
                download_request = true;
                console.log("loadVideoToBlob: ", video);
                //TODO eval if type is yt or custom and if custom use custom provided as filename
                await downloadBlob(video, onProgress)
                download_request = false;
            } 
        } catch (error) {
            download_request = false;
            //console.error('Error loading video to blob:', error);
        }
    }

    JeopardyBoardStore.subscribe(value => {
        if (value != null) {
            current = value.current;
            if ((current != null) && 
                (current.vector2d.x === question.vector2d.x && current.vector2d.y === question.vector2d.y)
            ) {
                match(question.question_type)
                .with({ Video: P.select() }, async (vid) => {
                    videoType = VideoPlayerType.VIDEO;
                        loadVideoToBlob(vid)
                })
                .with({ Youtube: P.select() }, async (aud) => {
                    videoType = VideoPlayerType.YOUTUBE;
                    createYouTubePlayer(); 
                })
                .otherwise(() => {
                    download_request = false;
                });
                if (blob === null)
                    return;
                ;
            }
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

    function createYouTubePlayer() : boolean {
        if (current == null) {
            return false;
        }
        if (player != null) {
            return true;
        }
        let result = false;
        match(current.question_type)
        .with({ Youtube: P.select() }, (data) => {
            //if element #player is not found, return false
            const playerElement = document.getElementById("player");
            console.log("?", playerElement);
            if (!playerElement) {
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

            player = new YouTubePlayerPlus(playerElement, options)
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




    function get_Blob_Type(): BlobType {
        return getBlobType(blob);
    }

    function get_blob(): Blob {
        try {
            if (blob == null) {
                throw new Error('Blob is null');
            }
            if (blob == undefined) {
                throw new Error('Blob is undefined');
            }
            return blob;
        } catch (error) {
            console.error('Error getting blob:', error);
            throw error;
        }
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
            <div class="overlay-content">
                {#if videoType == VideoPlayerType.VIDEO}
                    {#if blob != null}
                        {#if get_Blob_Type() == BlobType.IMAGE}
                            <ImageBlob image={get_blob()}/>
                        {:else if get_Blob_Type() == BlobType.VIDEO}
                            <VideoBlob video={get_blob()} currUserIsAdmin={isAdmin()}/>
                        {:else if get_Blob_Type() == BlobType.AUDIO}
                            <AudioBlob audio={get_blob()}/>
                        {:else if get_Blob_Type() == BlobType.TEXT}
                            <TextBlob text={get_blob()}/>
                        {:else}
                            <p>Unsupported file type</p>
                        {/if}
                    {:else}
                        <p>{file_download_progress?.current || 0} % /  {100} %  | Speed {file_download_progress?.speed}</p>
                    {/if}
                {:else if videoType == VideoPlayerType.YOUTUBE}
                    <!--edit hier with tailwind -->
                    <p id="player" class="player container mx-auto"></p>
                    <h1>${current.value}</h1>
                    <p>{current.question_type}</p>
                    <button on:click={() => player?.play()}>Play</button>
                    <button on:click={() => player?.pause()}>Pause</button>
                    <button on:click={() => player?.stop()}>Stop</button>
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