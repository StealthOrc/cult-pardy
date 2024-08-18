<script lang="ts">
	import { WebsocketStore } from "$lib/stores/WebsocketStore";
	import type { WebsocketServerEvents, WebsocketSessionEvent } from "cult-common";
	import { inflate } from "fflate";
	import type { WebSocketSubject } from "rxjs/webSocket";
	import { onMount } from "svelte";
	import { match, P } from "ts-pattern";

    export let video: Blob
    export let currUserIsAdmin: boolean = false;
    
    let ws: WebSocketSubject<WebsocketSessionEvent>;
    const wsStore : WebsocketStore | null = WebsocketStore;
    let player: HTMLVideoElement | null = null;
    let isPlayAllowed: boolean = false;
    let isPauseAllowed: boolean = false;

    //TODO: using a second websocket for video events is not ideal. We should be able to use the same websocket, from Boad, for all events
    if (wsStore != null) {
        wsStore.subscribe(value => {
            ws = value;
            ws.subscribe({
                next: (message) => {
                    if (message instanceof ArrayBuffer) {
                        let u8 = new Uint8Array(message);
                        inflate(u8, (err, infalte) => {
                            if (err) {
                                console.error('Deflation error:', err);
                            } else {
                                const decoder = new TextDecoder();
                                let json : string = decoder.decode(infalte);
                                const parsed : WebsocketServerEvents = JSON.parse(json);
                                const updated = handleVideoEvent(parsed);
                            }
                        });
                    };
                },
                error: (error) => {
                    console.log(error);
                    wsStore.stop();
                }
            });
        })
    }

    function handleVideoEvent(event: WebsocketServerEvents): boolean {
        match(event)
        .with({ ActionState : P.select()}, (data) => {
            match(data.Media)
            .with("Play", (data) => {
                console.log("ActionState: Play");
                if (!player)
                   return; 
                isPlayAllowed = true;
                try {
                    player.click();
                    player.play();
                }
                catch (error) {
                    console.error(error);
                }
            })
            .with("Pause", (data) => {
                console.log("ActionState: Pause");
                if (!player)
                    return;
                isPauseAllowed = true;
                player.pause();
                //TODO: get pause time to sync pause after we paused the player
            })
            .otherwise(() => {
                
            })
        })
        .otherwise(() => {
            //console.log("other event?!: ",event)
        })
        return true;
    }

    function onPlay(event) {
        console.log("onPlay: ", isPlayAllowed);
        if (!ws || !player)
            return;
        //if the play is coming from websocket then dont pause and dont send websocket message
        if (isPlayAllowed) 
        {
            isPlayAllowed = false;
            return;
        }
        //requesting play until then pause
        console.log("pausing because wait for ws");
        isPauseAllowed = true;
        player.pause();
        let playEvent: WebsocketSessionEvent = { VideoEvent: "Play" };
        ws.next(playEvent);
    }

    function onPause(event) {
        console.log("onPause", isPauseAllowed);
        if (!ws || !player)
            return;
        if (isPauseAllowed) 
        {
            isPauseAllowed = false;
            return;
        }
        //requesting pause at current time in microseconds
        let secs = player.currentTime * 1000000;
        let pauseEvent: WebsocketSessionEvent = { VideoEvent: {Pause: secs} };
        ws.next(pauseEvent);
    }

    onMount(() => {
        player = document.getElementById("player") as HTMLVideoElement;
    })

</script>


<video on:play={onPlay} on:pause={onPause} id="player" src={URL.createObjectURL(video)} controls={currUserIsAdmin}>
    <track kind="captions" />
</video>