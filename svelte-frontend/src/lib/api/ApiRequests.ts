
import {
    type ApiResponse,
    type CFile,
    type DiscordUser,
    type DTOFileChunk,
    type DTOFileData, type DTOFileToken,
    type FileDataReponse,
    type JeopardyBoard,
    type UserSessionId
} from "cult-common";
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


    const INFO_URL: string = 'api/info';
    const SESSION_DATA_URL: string = 'api/session-data';
    const AUTHORIZATION_URL: string = 'api/authorization';
    const DISCORD_SESSION_URL: string = 'api/discord_session';
    const JOIN_URL: string = 'api/join';
    const BOARD_URL: string = 'api/board';
    const FILEDATA_URL:string = 'api/upload/filedata';
    const FILECHUNK_URL:string = 'api/upload/filechunk';
    const GETFILE_URL:string = 'api/file/'; // add filename after the slash



export async function authorization(): Promise<ApiResponse> {
    const response : Response = await api_get_request(AUTHORIZATION_URL);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function discord_session(): Promise<DiscordUser> {
    const response : Response = await api_get_request(DISCORD_SESSION_URL);
    return await response.json();
}

export async function session_data(): Promise<SessionData> {
    const response : Response  = await api_get_request(SESSION_DATA_URL);
    const json = await response.json();
    const user_session_id: UserSessionId = json.user_session_id;
    const session_token: SessionToken = json.session_token;
    return new SessionData(user_session_id, session_token);
}




export async function join(): Promise<ApiResponse> {
    const response : Response = await api_get_request(JOIN_URL);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function board(): Promise<JeopardyBoard | null> {
    const response : Response  = await api_get_request(BOARD_URL);
    if (response == null || !response.ok) {
        return null;
    }
    return await response.json();
}


export async function UserInfo() {
    const discord = api_get_request(DISCORD_SESSION_URL);
    const auth = api_get_request(AUTHORIZATION_URL);

    const [discord_response, auth_response] = await Promise.all([discord, auth]);



    return {
        discord_response, 
        auth_response
    };

}

export async function upload_data(data:DTOFileData): Promise<FileDataReponse> {
    const response : Response = await api_post_request(FILEDATA_URL, data, "");
    if (response == null || !response.ok) {
        return {
            Failed: "Failed to upload data"
        }
    }
    return await response.json();
}

export async function upload_chunk(data:DTOFileChunk, token:DTOFileToken): Promise<Response> {
    return await api_post_request(FILECHUNK_URL, data, token.token);
}

export async function get_file(filename: string): Promise<CFile> {
    console.log("getting file:",GETFILE_URL + filename);
    const response: Response | null = await api_get_request(GETFILE_URL + filename);
    const json = await response.json();
    return json;
}

export async function api_post_request(url: string, data:unknown,token:string ): Promise<Response> {
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
        return await fetch(url + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}&file-token=${token}`, {
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
export async function api_get_request(url: string): Promise<Response> {
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
        return await fetch(url + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}`, {
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
