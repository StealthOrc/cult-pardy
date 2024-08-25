import { get_file, get_file2 } from "$lib/api/ApiRequests";
import { buildUint8ArrayFromChunks } from "$lib/BinaryConversion";
import { decompressData, formatSpeed } from "$lib/create/fileUploadUtils";
import type { LobbyId, MediaToken, NumberScope } from "cult-common";
import { XXH64 } from "xxh3-ts";

export enum BlobType {
    TEXT = "text",
    IMAGE = "image",
    AUDIO = "audio",
    VIDEO = "video",
    OTHER = "other",
    UNKNOWN = "unknown"
}



export function getBlobType(blob: Blob | null): BlobType {
    if (blob == null) {
        return BlobType.UNKNOWN;
    } else if (blob.type.startsWith("text")) {
        return BlobType.TEXT;
    } else if (blob.type.startsWith("image")) {
        return BlobType.IMAGE;
    } else if (blob.type.startsWith("audio")) {
        return BlobType.AUDIO;
    } else if (blob.type.startsWith("video")) {
        return BlobType.VIDEO;
    } else {
        return BlobType.OTHER;
    }
}

export type FileDownloadProgress = {
    current: number;
    speed: string;
    name: string 
    blob? : Blob;
    type?: BlobType;
    size?: number;
    upload_date?: string;
    uploader_id?: string;
    hash?: string;
};

export function test<T>(ob:T, s:string): boolean {
    return typeof ob === "object" && ob != null &&s in ob
}
export function test2<T extends object | string, K extends keyof T>(ob: T, s: string): T[K] | null {
    if (typeof ob === "object" && ob !== null && s in ob) {
        return ob[s as K];
    }
    return null;
}

export async function downloadBlob2(filename: string, range: NumberScope): Promise<Blob> {
    console.log("filename", filename);
    const response: Response = await get_file2(filename, range);
    if (!response.ok || !response.body || !(response.body instanceof ReadableStream)) {
        throw new Error("Failed to download file");
    }
    const file_type = response.headers.get('file-type') || 'video/mp4';


    const reader = response.body.getReader();
    const chunks: Uint8Array[] = [];
    let done = false;

    const startTime = performance.now();
    while (!done) {
        const { value, done: isDone } = await reader.read();
        if (value) {
            chunks.push(value);
        }
        if (isDone) {
            console.log("Download complete");
            done = true;
        }
    }
    console.log("Downloaded in", (performance.now() - startTime) / 1000, "seconds");
    const fileData = buildUint8ArrayFromChunks(chunks);
    console.log("fileData", fileData);
    const decompressedData = await decompressData(fileData);
    console.log("decompressedData", decompressedData);
    const blob = new Blob([decompressedData], { type: file_type });
    return blob
}



export async function downloadBlob(filename: string, lobby_id:LobbyId, media_token:MediaToken, onProgress: (progress: FileDownloadProgress) => void): Promise<void> {
        console.log("filename", filename);
        const response: Response = await get_file(filename, lobby_id,media_token);
        if (!response.ok || !response.body || !(response.body instanceof ReadableStream)) {
            throw new Error("Failed to download file");
        }

        const file_name = response.headers.get('file-name') || 'unknown';
        const file_type = response.headers.get('file-type') || 'video/mp4';
        const file_size : number = (response.headers.get('file-size') || 0) as number;
        const file_upload_date = response.headers.get('file-upload-date') || 'unknown';
        const uploader_id = response.headers.get('uploader-id') || 'unknown';
        const validate_hash = response.headers.get('validate-hash') || null;
        if (validate_hash == null) {
            throw new Error("validate_hash is null");   
        }
        onProgress({ current: 0, speed: "0 B/s", name: file_name, size: file_size, upload_date: file_upload_date, uploader_id: uploader_id});
        const reader = response.body.getReader();
        const chunks: Uint8Array[] = [];
        let downloadedSize = 0;
        let downloadPercentage = 0;
        let done = false;
        let end_speed = "0 B/s";
        const startTime = performance.now();
        while (!done) {
            const { value, done: isDone } = await reader.read();
            if (value) {
                chunks.push(value);
                downloadedSize += value.length; 
                downloadPercentage = Math.floor((downloadedSize / file_size) * 100);  
                const speed = formatSpeed(downloadedSize, (performance.now() - startTime) / 1000);
                onProgress({ current: Math.min(downloadPercentage, 100), speed: speed, name: file_name, size: file_size, upload_date: file_upload_date, uploader_id: uploader_id});
            }
            if (isDone) {
                console.log("Download complete");
                done = true;
                end_speed = formatSpeed(downloadedSize, (performance.now() - startTime) / 1000);
                onProgress({ current: 100, speed: end_speed, name: file_name, size: file_size, upload_date: file_upload_date, uploader_id: uploader_id});
                if (downloadedSize != file_size) {
                    console.error("downloadedSize != totalSize", downloadedSize, file_size);
                    throw new Error("downloadedSize != totalSize");
                }
            }
        }


        console.log("Downloaded in", (performance.now() - startTime) / 1000, "seconds");

        const fileData = buildUint8ArrayFromChunks(chunks);
        console.log("fileData", fileData);
        const hash = XXH64(Buffer.from(fileData)).toString()
        console.log("hash is valid", hash == validate_hash, hash, validate_hash);
        if (hash != validate_hash) {
            throw new Error("hash is not valid");
        }
        const decompressedData = await decompressData(fileData);
        console.log("decompressedData", decompressedData);

        const blob = new Blob([decompressedData], { type: file_type });
        
        onProgress({ current: 100, speed: end_speed, blob:blob, type: getBlobType(blob), name: file_name, size: file_size, upload_date: file_upload_date, uploader_id: uploader_id, hash: hash});

}