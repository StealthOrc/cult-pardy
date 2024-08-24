<script lang="ts">
	import { CONST } from "$lib/const";
	import { get_global_time } from "$lib/lib";
	import { mediaPlayerContextStore } from "$lib/stores/MediaPlayerStore";
	import { mediaStateStore, type MediaPlayerSessionType } from "$lib/stores/MediaStateStore";
	import type { BoardContext, MediaPlayerContext } from "$lib/types";
	import type { MediaState, WebsocketSessionEvent } from "cult-common";
	import { getContext, onMount, setContext } from "svelte";
	import JeopardyBoard from "../JeopardyBoard.svelte";
	import { JeopardyBoardStore } from "$lib/stores/JeopardyBoardStore";
	import JeopardyCategory from "../JeopardyCategory.svelte";
	import { match, P } from "ts-pattern";
	import type { EventHandler } from "svelte/elements";
	import { downloadBlob2 } from "./blodUtils";
	import { WebsocketStore } from "$lib/stores/WebsocketStore";

    export let name = "world";
    export let currUserIsAdmin: boolean = false;
    let player: HTMLVideoElement | null = null;
    

    onMount(() => {
        loadVideo();
        player = document.getElementById("player") as HTMLVideoElement;
        if (player == null) return;
        console.log("TEK ONLOADING!!!!!", player);
        match(JeopardyBoardStore.getActionState())
        .with({MediaPlayer: P.select()}, (data) => {
            if (!player) return
            if (data.playing) determinePlayAndSeekPlay(data);
            else determinePauseAndPauseSeek(data);
            ignore = true;
            mediaStateStore.setMediaState(data);
        })
        
        
    })
    let ignore = false
    let media: MediaPlayerSessionType;
    mediaStateStore.subscribe(value => {
        media = value;
        if(value == null || value.mediaState == null || ignore) {
            ignore = false;
            return;
        };
        if (player == null) {
            JeopardyBoardStore.setActionState({MediaPlayer: value.mediaState});
        } else {
            doMediaStateChange(value.mediaState);
        }
    })


    enum EventStatusEnum {
        NONE = "NONE",
        PLAY = "PLAY",
        PAUSE = "PAUSE",
        START_SEEKING = "START_SEEKING",//MAYBE
        START_SEEKING_PLAY = "START_SEEKING_PLAY",//MAYBE
        PAUSE_START_SEEKING = "PAUSE_START_SEEKING",//MAYBE
        SEEKING = "SEEKING",
        SEEKING_PLAY = "SEEKING_PLAY",
    }


    let status = EventStatusEnum.NONE;
    let eventSourceLog = Array<EventStatusEnum>();
        
    let store = $WebsocketStore;

    enum StateUpdateType {
        PLAY = "PLAY",
        PAUSE = "PAUSE",
        SEEK = "SEEK",
        USER = "USER",
        WEBSOCKET = "WEBSOCKET",
        STARTING_SEEKING = "START_SEEKING",
        STOPING_SEEKING = "STOP_SEEKING",
        WEbSOCKET_SEEKING = "WEBSOCKET_SEEKING",
        
    }

  


    function handleEvent(state: MediaState): boolean {
        if (player == null) return false;
        if (state.interaction_id.id === store.websocket_id.id) {
            let proposed_time;
            let isSeeking;
            if (state.playing){
                proposed_time = (get_global_time(media.correction) - state.global_timestamp) / 1000 + state.video_timestamp; // Video timestamp from global timestamp from the server 
                isSeeking = Math.abs(proposed_time - player.currentTime) > CONST.PLAYING_THRESH;
            } else {
                proposed_time =  state.video_timestamp; // Video timestamp from the video itself
                isSeeking = (Math.abs(player.currentTime - proposed_time) > CONST.PAUSED_THRESH);
            }
            if(isSeeking){
                console.log("SEEKING", state);
                return false;
            }

            return false;
        }
        return true;
    }


    async function doMediaStateChange(state: MediaState) {
        try {
            if (!player) return;
                if (!handleEvent(state)) return;
                if (state.playing) {
                    determinePlayAndSeekPlay(state);
                } else {
                    determinePauseAndPauseSeek(state);
                }
        } catch (error) {
            console.error('Error changing media state:', error);
        }
    }
    


    async function pause() {
        if (!player)
            return;
        if (status == EventStatusEnum.PAUSE || status == EventStatusEnum.PAUSE_START_SEEKING) {
            update_status(StateUpdateType.PAUSE)
            return;
        }
        if(status != EventStatusEnum.NONE){
            return;
        }

        const state = currentMediaState(false);
        requestPlayerChangeState(state);
    }

    async function play(){
        if (!player)
            return;
        if (status == EventStatusEnum.PLAY) {
            update_status(StateUpdateType.PLAY)
            return;
        }
        if(status != EventStatusEnum.NONE){
            return;
        }
        
        const state = currentMediaState(true);
        requestPlayerChangeState(state);
    }

    async function seek() {
        if (!player)
            return;
        if (status == EventStatusEnum.START_SEEKING || status == EventStatusEnum.START_SEEKING_PLAY) {
            update_status(StateUpdateType.STARTING_SEEKING);
            return;
        }
        if (status == EventStatusEnum.SEEKING || status == EventStatusEnum.SEEKING_PLAY) {
            return;
        }
        if (status != EventStatusEnum.NONE) {
            return;
        }

    }


    const validStatuses = {
        [EventStatusEnum.PLAY]: EventStatusEnum.NONE,
        [EventStatusEnum.PAUSE]: EventStatusEnum.NONE,
        [EventStatusEnum.SEEKING]: EventStatusEnum.NONE,
        [EventStatusEnum.PAUSE_START_SEEKING]: EventStatusEnum.START_SEEKING,
        [EventStatusEnum.START_SEEKING_PLAY]: EventStatusEnum.SEEKING_PLAY,
        [EventStatusEnum.START_SEEKING]: EventStatusEnum.SEEKING,
        [EventStatusEnum.SEEKING_PLAY]: EventStatusEnum.PLAY,
        [EventStatusEnum.NONE]: EventStatusEnum.NONE,
    };



    async function update_status(updae: StateUpdateType) {
        let old_status = status;
        let new_status = get_new_status(old_status);
        set_status(new_status,updae);
    }

    function get_new_status(old_status:EventStatusEnum): EventStatusEnum {
        let test =  validStatuses[old_status] || EventStatusEnum.NONE;
        return test
    }


    function set_status(new_status: EventStatusEnum, update: StateUpdateType) {
        console.log("UPDATE STATUS FROM", status, "TO", new_status , "WITH", update);
        status = new_status;
        eventSourceLog.push(new_status);
    }   
        



    function onEnded(event: any) {
        if (!player)
            return;
        try {
            player.load();
        } catch (error) {
            console.error('Error resetting video:', error);
        }
        const state = currentMediaState(false);
        requestPlayerChangeState(state);
    }



    async function seekToTime(time: number): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            if (!player) return;
            const timer = setTimeout(() => {
                if (!player) return;
                player.removeEventListener('seeked', onSeeked);
                reject(new Error('Seek operation timed out after 5 seconds'));
            }, 2000);
            function onSeeked() {
                if (!player) return;
                clearTimeout(timer);
                player.removeEventListener('seeked', onSeeked);
                resolve();
            }
            player.addEventListener('seeked', onSeeked);
            player.currentTime = time;
            console.log("SEEKING TO", time);
        });
    }


    
    async function determinePlayAndSeekPlay(mediaState: MediaState) {
        if (!player) return;
        console.log()
        let proposed_time = (get_global_time(media.correction) - mediaState.global_timestamp) / 1000 + mediaState.video_timestamp; // Video timestamp from global timestamp from the server 
        let isSeeking = Math.abs(proposed_time - player.currentTime) > CONST.PLAYING_THRESH;
        let isPlaying = !player.paused;
        if(status != EventStatusEnum.SEEKING){
            if(isSeeking ){
                set_status(isPlaying ? EventStatusEnum.START_SEEKING : EventStatusEnum.START_SEEKING_PLAY, StateUpdateType.WEbSOCKET_SEEKING); // SEEKING# SEEKING_PLAY
                await seekToTime(proposed_time);
            } else {
                set_status(isPlaying ? EventStatusEnum.NONE : EventStatusEnum.PLAY,StateUpdateType.WEBSOCKET); // PLAY# NONE
            }
        }   
        if (!isPlaying) player.play()
    }




    async function determinePauseAndPauseSeek(mediaState: MediaState) {
        if (!player) return;    
        let isPause = player.paused;
        let proposed_time =  mediaState.video_timestamp; // Video timestamp from the video itself
        let isSeeking = (Math.abs(player.currentTime - proposed_time) > CONST.PAUSED_THRESH);  
        if(status != EventStatusEnum.SEEKING){
            if (isSeeking) {
                set_status(isPause ? EventStatusEnum.START_SEEKING : EventStatusEnum.PAUSE_START_SEEKING, StateUpdateType.WEbSOCKET_SEEKING); // SEEKING# PAUSE_SEEKING
            } else {
                set_status(isPause ? EventStatusEnum.NONE : EventStatusEnum.PAUSE,StateUpdateType.WEBSOCKET); // NONE# PAUSE
            }
        }
        if (!isPause) player.pause()
        if (isSeeking) await seekToTime(proposed_time);
        
    }



    
    function currentMediaState(playing:boolean):MediaState{
        return {
            video_timestamp: player?.currentTime || 0,
            last_updated: get_global_time(media.correction),
            global_timestamp: get_global_time(media.correction),
            playing,
            interaction_id: {
                id: store.websocket_id.id
            },
        }
    }


    function endSeeking() {
        if (!player)
            return;
        if (status == EventStatusEnum.SEEKING || status == EventStatusEnum.SEEKING_PLAY || status == EventStatusEnum.START_SEEKING) {
            update_status(StateUpdateType.STOPING_SEEKING);
            return;
        }


        let playing = !player.paused;
        const state = currentMediaState(playing);
        requestPlayerChangeState(state);



    }


    function requestPlayerChangeState(state: MediaState): boolean {
        let changeStateEvent: WebsocketSessionEvent = { VideoEvent: {ChangeState: state} };
        store.webSocketSubject.next(changeStateEvent);
        return true;
    }



    async function loadVideo(start = 0) {
        if (!player) return;
        let headers = {};
        
        // If end is not specified, assume you want to load 10 MB chunks
     
        const end  = start + (10 * 1024 * 1024) - 1;  // 10 MB range
        let range = {start, end};
        //let blob = await downloadBlob2(name, range); 
        //player.src = URL.createObjectURL(blob);
    }



</script>





<div>
    <h1>status: {status}</h1>

</div>
<video 
bind:this={player}
on:play={play} on:pause={pause} on:ended={onEnded} 
on:seeking={seek}
on:seeked={endSeeking}
id="player" controls={currUserIsAdmin} muted>
    <track kind="captions" />
</video>