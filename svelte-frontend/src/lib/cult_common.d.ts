/* tslint:disable */
/* eslint-disable */
export interface LobbyId {
    id: string;
}

export interface WebsocketSessionId {
    id: string;
}

export type WebsocketError = { LobbyNotFound: LobbyId } | { SessionNotFound: UserSessionId } | { GameStarted: LobbyId } | "NotAuthorized" | "WebsocketCrashed" | { UNKNOWN: string };

export type WebsocketSessionEvent = { Click: Vector2D } | "Back" | { AddUserSessionScore: UserSessionId };

export type SessionEvent = { CurrentSessions: DTOSession[] } | { SessionJoined: DTOSession } | { SessionDisconnected: UserSessionId };

export type WebsocketEvent = { WebsocketJoined: WebsocketSessionId } | { WebsocketDisconnected: WebsocketSessionId };

export type BoardEvent = { CurrentBoard: DtowJeopardyBoard } | { CurrentQuestion: [Vector2D, DtoQuestion] } | { UpdateCurrentQuestion: Vector2D | null } | { UpdateSessionScore: [UserSessionId, number] };

export type WebsocketServerEvents = { Board: BoardEvent } | { Websocket: WebsocketEvent } | { Session: SessionEvent } | { Error: WebsocketError } | { Text: string };

export type QuestionType = { Media: string } | "Question";

export interface Question {
    question_type: QuestionType;
    question: string;
    value: number;
    answer: string;
}

export interface JsonPrinter {
    results: Record<string, boolean>;
}

export interface ApiResponse {
    success: boolean;
}

export interface DiscordID {
    id: string;
}

export interface UserSessionId {
    id: string;
}

export interface Category {
    title: string;
    questions: Question[];
}

export interface DtoQuestion {
    question_type: QuestionType;
    question_text: string | null;
    value: number;
    answer: string | null;
    won_user_id: UserSessionId | null;
}

export interface DtoCategory {
    title: string;
    questions: DtoQuestion[];
}

export interface Vector2D {
    x: number;
    y: number;
}

export interface DtoJeopardyBoard { 
    creator: UserSessionId;
    categories: DtoCategory[];
    current: Vector2D | null;
}

export interface JeopardyBoard {
    title: string;
    categories: Category[];
}

export type LobbyCreateResponse = { Created: LobbyId } | { Error: string };

export type JeopardyMode = "SHORT" | "NORMAL" | "LONG";

export interface DiscordUser {
    discord_id: DiscordID;
    username: string;
    avatar_id: string;
    discriminator: string;
    global_name: string;
}

export interface DTOSession {
    user_session_id: UserSessionId;
    score: number;
    discord_user: DiscordUser | null;
    is_admin: boolean;
}


export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
