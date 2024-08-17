
import {
    type ApiResponse,
    type DiscordUser,
    type JeopardyBoard,
    type UserSessionId
} from "cult-common";
import { CookieStore, type SessionCookies } from "$lib/stores/cookies";
import {CONST} from "$lib/const"

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


//    const INFO_URL: string = 'api/info';


export enum RequestContentType {
    FORM_DATA = "multipart/form-data",
    JSON = "application/json",
    OCTET_STREAM = "application/octet-stream"  
}


export async function authorization(): Promise<ApiResponse> {
    const response : Response = await api_get_request(CONST.AUTHORIZATION_URL, RequestContentType.JSON);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function discord_session(): Promise<DiscordUser> {
    const response : Response = await api_get_request(CONST.DISCORD_SESSION_URL, RequestContentType.JSON);
    return await response.json();
}

export async function session_data(): Promise<SessionData> {
    const response : Response  = await api_get_request(CONST.SESSION_DATA_URL, RequestContentType.JSON);
    const json = await response.json();
    const user_session_id: UserSessionId = json.user_session_id;
    const session_token: SessionToken = json.session_token;
    return new SessionData(user_session_id, session_token);
}




export async function join(): Promise<ApiResponse> {
    const response : Response = await api_get_request(CONST.JOIN_URL, RequestContentType.JSON);
    if (response == null || !response.ok) {
        return {success: false};
    }
    return await response.json();
}

export async function board(): Promise<JeopardyBoard | null> {
    const response : Response  = await api_get_request(CONST.BOARD_URL, RequestContentType.JSON);
    if (response == null || !response.ok) {
        return null;
    }
    return await response.json();
}


export async function UserInfo() {
    const discord = api_get_request(CONST.DISCORD_SESSION_URL, RequestContentType.JSON);
    const auth = api_get_request(CONST.AUTHORIZATION_URL, RequestContentType.JSON);

    const [discord_response, auth_response] = await Promise.all([discord, auth]);



    return {
        discord_response, 
        auth_response
    };

}

export async function get_file(filename: string): Promise<Response> {
    const headers = new Headers();
    headers.append("file-name", filename);
    return await api_get_request(CONST.GETFILE_URL, RequestContentType.OCTET_STREAM, headers);
}


export async function upload_file_part(file:FormData): Promise<Response> {
    return await api_post_formrequest(CONST.FILEPART, file, RequestContentType.FORM_DATA);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function objectToFormData(obj: Record<string, any>, formData = new FormData(), parentKey = ''): FormData {
    for (const [key, value] of Object.entries(obj)) {
        const formKey = parentKey ? `${parentKey}[${key}]` : key;

        if (value instanceof Blob || value instanceof File) {
            formData.append(formKey, value);
        } else if (value instanceof Array) {
            value.forEach((item, index) => {
                objectToFormData({ [index]: item }, formData, formKey);
            });
        } else if (typeof value === 'object' && value !== null) {
            objectToFormData(value, formData, formKey);
        } else {
            formData.append(formKey, String(value));
        }
    }
    return formData;
}


export async function api_post_request(url: string, data:unknown,token:string, type: RequestContentType): Promise<Response> {
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
                'Content-Type': type
            },
            body: JSON.stringify(data),
        });

        // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e ) {
        throw new Error("Failed to fetch");
    }
}


export async function api_post_formrequest(url: string, form:FormData, type: RequestContentType ): Promise<Response> {
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
       
        // POST Rquest with form data
        return await fetch(url + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}?file-name=FlyHigh3.mp4`, {
            method: 'POST',
            headers: {
                'Content-Type': type
            },
            body: form,
        });



        // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e ) {
        throw new Error("Failed to fetch");
    }
}



export async function api_get_request(url: string, type: RequestContentType, headers: Headers = new Headers()): Promise<Response> {
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
        headers.append("Content-Type", type);
        return await fetch(url + `?user-session-id=${cookies.userSessionId.id}&session-token=${cookies.sessionToken}`, {
            method: 'GET',
            headers: headers,
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

