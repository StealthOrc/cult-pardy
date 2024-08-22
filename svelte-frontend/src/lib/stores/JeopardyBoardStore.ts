import { dev } from "$app/environment";
import type { ActionState, DtoJeopardyBoard, DtoQuestion } from "cult-common";
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
        store.set(board);        
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
            board.action_state = state;
            return board;
        });       
    }

    function getActionState() : ActionState {
        let state: ActionState = "None";
        store.update((board) => {
            if (board == null) {
                return board;
            }
            state = board.action_state;
            return board;
        });
        return state;
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
        getActionState,
        subscribeActionState,
    }
}

