
import type { DtoJeopardyBoard, DtoQuestion } from "cult-common";
import { writable, type Subscriber, type Unsubscriber} from "svelte/store"; 


export const JeopardyBoardStore = createJeopardyBoardStore();


function createJeopardyBoardStore() {

    const current_board = writable<DtoJeopardyBoard|null>(null);


    function setBoard(board: DtoJeopardyBoard) {
        current_board.set(board);        
    }

    function setCurrent(current: DtoQuestion) {
        current_board.update((board) => {
            if (board == null) {
                return board;
            }
            board.current = current;
            return board;
        });       
    }


    function subscribe(this: void, run: Subscriber<DtoJeopardyBoard | null>): Unsubscriber {
        return current_board.subscribe(run);
    }

    
    
    return {
        current_board,
        setCurrent,
        setBoard,
        subscribe,
    }
}
