import { dev } from "$app/environment";
import type { ActionState, ActionStateType, DtoJeopardyBoard, DtoQuestion, UserSessionId } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 


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

    function setBoard(board: DtoJeopardyBoard) {
        console.log("Setting board");
        store.set(board);        
    }

    function setCurrent(current: DtoQuestion) {
        console.log("Setting current");
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
            board.action_state = state;
            return board;
        });       
    }


    function setActionStateType(state: ActionStateType) {
        store.update((board) => {
            if (board == null) {
                return board;
            }
            board.action_state.state = state;
            return board;
        });       
    }


    function getActionState() : ActionStateType {
        let state: ActionStateType = "None";
        store.update((board) => {
            if (board == null) {
                return board;
            }
            state = board.action_state.state;
            return board;
        });
        return state;
    } 


    function addMediaDownloadComplete(UserSessionId: UserSessionId) {
        store.update((board) => {
            if (board == null) {
                return board;
            }
            if (board.action_state != undefined) {
                if (board.action_state.current_type != undefined) {
                    if (typeof board.action_state.current_type === "object" && "Media" in board.action_state.current_type) {
                        board.action_state.current_type.Media.media_loaded.push(UserSessionId);
                    }
                }
            }   
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
    



    function subscribe(this: void, run: Subscriber<DtoJeopardyBoard | null>): Unsubscriber {
        return store.subscribe(run);
    }

    return {
        store,
        setCurrent,
        setActionState,
        setBoard,
        subscribe,
        addMediaDownloadComplete,
        setActionStateType,
        getActionState,
        subscribeActionState,
    }
}

