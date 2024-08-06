
import { type ApiResponse, type DiscordUser, type DTOFileChunk, type DTOFileData, type JeopardyBoard, type UserSessionId } from "cult-common";
import { CookieStore, type SessionCookies } from "$lib/stores/cookies";

/*
/api/info
/api/session
/api/authorization
/api/discord_session
/api/create
/api/join
/api/board
*/

let cookies : SessionCookies | null = null;

let updater : boolean = false;


enum BackendApiRequests {
    INFO = 'api/info',
    SESSION_DATA = 'api/session-data',
    AUTHORIZATION = 'api/authorization',
    DISCORD_SESSION = 'api/discord_session',
    //CREATE = 'api/create',
    JOIN = 'api/join',
    BOARD = 'api/board',
    FILEDATA = 'api/upload/filedata',
    FILECHUNK = 'api/upload/filechunk',
    
}   



export async function authorization(): Promise<ApiResponse> {
    const response : Response | null = await api_get_request(BackendApiRequests.AUTHORIZATION);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function discord_session(): Promise<DiscordUser | null> {
    const response : Response | null = await api_get_request(BackendApiRequests.DISCORD_SESSION);
    if (response == null || !response.ok) {
        return null;
    }
    return await response.json();
}

export async function session_data(): Promise<SessionData | null> {
    const response : Response | null = await api_get_request(BackendApiRequests.SESSION_DATA);
    if (response == null || !response.ok) {
        return null;
    }
    const json = await response.json();
    const user_session_id: UserSessionId = json.user_session_id;
    const session_token: SessionToken = json.session_token;
    return new SessionData(user_session_id, session_token);
}




export async function join(): Promise<ApiResponse> {
    const response : Response | null = await api_get_request(BackendApiRequests.JOIN);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function board(): Promise<JeopardyBoard | null> {
    const response : Response | null = await api_get_request(BackendApiRequests.BOARD);
    if (response == null || !response.ok) {
        return null;
    }
    return await response.json();
}


export async function UserInfo() {
    const discord = api_get_request(BackendApiRequests.DISCORD_SESSION);
    const auth = api_get_request(BackendApiRequests.AUTHORIZATION);

    const [discord_response, auth_response] = await Promise.all([discord, auth]);



    return {
        discord_response, 
        auth_response
    };

}

export async function upload_data(data:DTOFileData): Promise<ApiResponse> {
    const response : Response | null = await api_post_request(BackendApiRequests.FILEDATA, data);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function upload_chunk(data:DTOFileChunk): Promise<ApiResponse> {
    const response : Response | null = await api_post_request(BackendApiRequests.FILECHUNK, data);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}


export async function api_post_request(request:BackendApiRequests, data:unknown): Promise<Response | null> {
    try {
        if (!updater) {
            CookieStore.subscribe((c) => {
                cookies = c;
            });
            updater = true;
        }
        if (cookies == null) {
            return null;
        }  
        return await fetch(request + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data),
        });

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e ) {
        return null
    }
}
export async function api_get_request(request:BackendApiRequests): Promise<Response | null> {
    try {
        if (!updater) {
            CookieStore.subscribe((c) => {
                cookies = c;
            });
            updater = true;
        }
        if (cookies == null) {
            return null;
        }  
        return await fetch(request + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            },
            //credentials: 'include'
        });

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e ) {
        return null
    }
}


export class SessionData {
    public user_session_id: UserSessionId;
    public session_token: SessionToken;
    constructor(user_session_id: UserSessionId, session_token: SessionToken) {
        this.user_session_id = user_session_id;
        this.session_token = session_token;
    }
}



import { DateTime } from 'ts-luxon';

export class SessionToken {
    public token: string;
    public create: DateTime;

    constructor(token: string, create: DateTime) {
        this.token = token;
        this.create = create;
    }
}
