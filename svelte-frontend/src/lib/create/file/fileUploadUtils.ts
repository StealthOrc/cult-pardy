import { inflate, deflate } from 'fflate';
import { XXH64 } from 'xxh3-ts';
import { Buffer } from 'buffer';
import { CONST } from '$lib/const';

export type FileUploadProgress = {
    current: number;
    speed: string;
};

export async function handleBoardFileUpload(file: File, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const xhr = new XMLHttpRequest();

    xhr.open('POST', CONST.CREATE_LOBBY_URL, true);
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.onload = function() {
        if (xhr.status === 200) {
            console.log('File uploaded successfully');
        } else {
            console.error('Failed to upload file');
        }
    }  
    const startTime = performance.now();
    xhr.upload.onprogress = function(event) {
        if (event.lengthComputable) {
            const progress = Math.min(Math.ceil(((event.loaded) / event.total) * 100), 100);
            const speed = formatSpeed(event.loaded, (performance.now() - startTime) / 1000);
            onProgress({ current: progress, speed});
        }
    }
    xhr.onerror = function() {
        console.error('Failed to upload chunk');
    }

    const reader = new FileReader();
    reader.onload = async function(event) {
        const json = event.target?.result;
        xhr.send(json);
    }

    reader.readAsText(file);
}
export async function handleMediaFileUpload(file: File, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const arrayBuffer = await file.arrayBuffer(); 
    const deflatedData = await compressData(arrayBuffer);
    const hash = XXH64(Buffer.from(deflatedData)).toString()

    const blob2 = new Blob([deflatedData], { type: file.type });

    const form = new FormData();
    form.append('file_data', blob2);
    form.append('validate_hash', hash);
    
    const xhr = new XMLHttpRequest();

    xhr.open('POST', "api/upload/filepart", true);
    xhr.setRequestHeader("file-name", file.name);
    xhr.onload = function() {
        if (xhr.status === 200) {
            console.log('Chunk uploaded successfully');
        } else {
            console.error('Failed to upload chunk');
        }
    }  
    const startTime = performance.now();
    xhr.upload.onprogress = function(event) {
        if (event.lengthComputable) {
            const progress = Math.min(Math.ceil(((event.loaded) / event.total) * 100), 100);
            const speed = formatSpeed(event.loaded, (performance.now() - startTime) / 1000);
            onProgress({ current: progress, speed});
        }
    }



    xhr.onerror = function() {
        console.error('Failed to upload chunk');
    }

    xhr.send(form);

}


export async function compressData(data: ArrayBuffer): Promise<ArrayBuffer> {
    return new Promise((resolve, reject) => {
        deflate(new Uint8Array(data), (err, compressedData) => {
            if (err) {
                reject(err);
            } else {
                resolve(compressedData.buffer); // Return the compressed data as an ArrayBuffer
            }
        });
    });
}

export async function decompressData(data: ArrayBuffer): Promise<ArrayBuffer> {
    return new Promise((resolve, reject) => {
        inflate(new Uint8Array(data), (err, decompressedData) => {
            if (err) {
                reject(err);
            } else {
                resolve(decompressedData.buffer); // Return the compressed data as an ArrayBuffer
            }
        });
    });
}

export function formatSpeed(bytes: number, seconds: number) {
    if (seconds <= 0) return '0 MB/s';
    const megabytes = bytes / (1024 * 1024);
    const speed = megabytes / seconds;
    return `${speed.toFixed(2)} MB/s`;
}