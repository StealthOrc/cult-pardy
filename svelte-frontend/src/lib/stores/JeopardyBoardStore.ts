import { dev } from "$app/environment";
import type { ActionState, DtoJeopardyBoard, DtoQuestion } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 
import { match, P } from "ts-pattern";
import { mediaStateStore } from "./MediaStateStore";

export const JeopardyBoardStore = createJeopardyBoardStore();

if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.JeopardyBoardStore.store = JeopardyBoardStore.store;
            }
        });
    }
}

function createJeopardyBoardStore() {

    const store = writable<DtoJeopardyBoard|null>(null);

    function setBoard(board: DtoJeopardyBoard){
        if (board != null) {
            console.log("setActionState: ", board.action_state);
            match(board.action_state)
            .with({MediaPlayer: P.select()}, (media) => {
                mediaStateStore.setMediaStatus(media.status);
            }).otherwise(() =>
                mediaStateStore.resetMedia()
            );

        }


        store.set(board);        
        console.log("setBoard !!!!!!", board);
    }

    function setCurrent(current: DtoQuestion) {
        store.update((board) => {
            if (board == null) {
                return board;
            }
            board.current = current;
            return board;
        });       
    }

    function setActionState(state: ActionState) {
        store.update((board) => {
            if (board == null) {
                return board;
            }
            console.log("setActionState2    : ", state);
            match(state)
            .with({MediaPlayer: P.select()}, (media) => {
                mediaStateStore.setMediaStatus(media.status);
            }).otherwise(() =>
                mediaStateStore.resetMedia()
            );




            board.action_state = state;
            return board;
        });       
    }




    function subscribeActionState(this: void, run: Subscriber<ActionState>): Unsubscriber {
        return store.subscribe((data) => {
            if (data != null) {
                run(data.action_state);
            }   
        });
    }


    function getCurrentMediaID(this: void) : number {
        let media: number = 0;
        store.update((board) => {
            if (board == null) {
                return board;
            }
            if (board.current == null) {
                return board;
            }
            match(board.action_state)
                .with({MediaPlayer: P.select()}, (player) => {
                    media = player.current_media;
                }
            )
            return board;
        });
        return media;
    }
    



    function subscribe(this: void, run: Subscriber<DtoJeopardyBoard | null>): Unsubscriber {
        return store.subscribe(run);
    }

    return {
        store,
        setCurrent,
        setActionState,
        setBoard,
        subscribe,
        getCurrentMediaID,
        subscribeActionState,
    }
}

