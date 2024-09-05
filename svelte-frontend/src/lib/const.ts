const CONST ={
        SESSION_DATA_URL: '/api/session-data',
        AUTHORIZATION_URL: '/api/authorization',
        DISCORD_SESSION_URL:'api/discord_session',
        JOIN_URL: '/api/join',
        BOARD_URL: '/api/board',
        FILEPART: '/api/file/upload',
        GETFILE_URL: '/api/file/download',
        CREATE_LOBBY_URL: '/api/create',
        FILE_LIST_URL: '/api/file/list',
        FILES_URL: '/api/files',
        //Contexts
        MEDIAPLAYERCTX: 'mediaplayer',
        BOARDCTX: 'board',
        num_time_sync_cycles: 10,
        PLAYING_THRESH: 1,
        THRESH: 0.05,
        PAUSED_THRESH: 0.01,
    }

 export { CONST };



    
export enum FileType {
    PNG = '/_svelte_kit_assets/assets/icons/png.png',
    UNKNOWN = '/_svelte_kit_assets/icons/unknow.png',
    PDF = '/_svelte_kit_assets/icons/pdf.png',
    MP4 = '/_svelte_kit_assets/icons/mp4.png',
    JPG = '/_svelte_kit_assets/icons/jpg.png'
}

export function getFileType(file_name: string): string {
    const extension = getFileExtension(file_name);
    return FileType[extension as keyof typeof FileType] || FileType.UNKNOWN;
}

function getFileExtension(filetype: string): string {
    const parts = filetype.split('/');
    return parts.length > 1 ? parts.pop()!.toUpperCase() : '';
}