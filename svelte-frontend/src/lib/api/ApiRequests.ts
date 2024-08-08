
import { type ApiResponse, type DiscordUser, type DTOFileChunk, type DTOFileData,type DTOFileToken,type FileDataReponse,type JeopardyBoard, type UserSessionId } from "cult-common";
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
    const response : Response  = await api_get_request(BackendApiRequests.AUTHORIZATION);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function discord_session(): Promise<DiscordUser> {
    const response : Response = await api_get_request(BackendApiRequests.DISCORD_SESSION);
    if (response == null || !response.ok) {
        throw new Error("Failed to fetch discord session");
    }
    return await response.json();
}

export async function session_data(): Promise<SessionData> {
    const response : Response = await api_get_request(BackendApiRequests.SESSION_DATA);
    const json = await response.json();
    const user_session_id: UserSessionId = json.user_session_id;
    const session_token: SessionToken = json.session_token;
    return new SessionData(user_session_id, session_token);
}




export async function join(): Promise<ApiResponse> {
    const response : Response = await api_get_request(BackendApiRequests.JOIN);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function board(): Promise<JeopardyBoard> {
    const response : Response= await api_get_request(BackendApiRequests.BOARD);
    if (response == null || !response.ok) {
        throw new Error("Failed to fetch board");
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

export async function upload_data(data:DTOFileData): Promise<FileDataReponse> {
    console.log("STARTING data");
    const response : Response | null = await api_post_request(BackendApiRequests.FILEDATA, data, "");
    if (response == null || !response.ok) {
        return {
            Failed: "Failed to upload data"
        }
    }
    return await response.json();
}

export async function upload_chunk(data:DTOFileChunk, token:DTOFileToken): Promise<Response> {
    return await api_post_request(BackendApiRequests.FILECHUNK, data, token.token);

}


export async function api_post_request(request:BackendApiRequests, data:unknown, addon:string): Promise<Response> {
    try {
        if (!updater) {
            CookieStore.subscribe((c) => {
                cookies = c;
            });
            updater = true;
        }
        if (cookies == null) {
            throw new Error("No cookies");
        }  

        return await fetch(request  + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}&file-token=${addon}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data),
        });

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e ) {
        throw new Error("Failed to fetch");
    }
}
export async function api_get_request(request:BackendApiRequests): Promise<Response> {
    try {
        if (!updater) {
            CookieStore.subscribe((c) => {
                cookies = c;
            });
            updater = true;
        }
        if (cookies == null) {
            throw new Error("No cookies");
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
        throw new Error("Failed to fetch");
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
