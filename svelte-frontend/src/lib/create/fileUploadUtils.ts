import { inflate, deflate } from 'fflate';
import { XXH64 } from 'xxh3-ts';
import { Buffer } from 'buffer';
import type { DTOFileChunk, DTOFileData, DTOFileToken, FileChunkHash, FileDataForm, FileDataReponse } from 'cult-common';
import { upload_chunk, upload_chunk2, upload_chunk3, upload_data } from '$lib/api/ApiRequests';
import { match, P } from 'ts-pattern';  
import axios from 'axios';


const CHUNK_SIZE = 25_000; // 200 KB 
const CHUNK_SIZE2 = 250_000; 
const MAX_PARALLEL_UPLOADS = 5; // Maximum number of parallel uploads
const RETRY_LIMIT = 3; // Number of retry attempts for failed chunks


export type FileUploadProgress = {
    loaded: number;
    total: number;
    speed: string;
};


export enum FileUploadType {
    V1 = "V1",
    V2 = "V2",
    V3 = "V3",
    WS1 = "WS1",
    WS2 = "WS2"
}


export async function handleFileUpload(file: File, type:FileUploadType, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const arrayBuffer = await file.arrayBuffer(); 

    // Compress the file data asynchronously
    const deflatedData = await compressData(arrayBuffer);

    // Calculate hash of the deflated data
    const validateHash = XXH64(Buffer.from(deflatedData)).toString();
    const totalChunks = Math.ceil(deflatedData.byteLength / CHUNK_SIZE);
    onProgress({ loaded: 0, total: totalChunks, speed: '0 MB/s' });

    // Create chunks and calculate their hashes
    const fileChunks: DTOFileChunk[] = [];
    const chunkHashes: FileChunkHash[] = [];
    for (let i = 0; i < totalChunks; i++) {
        const start = i * CHUNK_SIZE;
        const end = Math.min((i + 1) * CHUNK_SIZE, deflatedData.byteLength);
        const chunkArray = deflatedData.slice(start, end);

        // Calculate hash for the chunk
        const chunkHash = XXH64(Buffer.from(chunkArray)).toString();
        chunkHashes.push({ hash: chunkHash });

        // Prepare chunk data
        fileChunks.push({
            file_name: file.name,
            index: i,
            chunk: chunkArray,
            validate_hash: { hash: chunkHash }
        });
    }

    // Prepare metadata
    const uploadData: DTOFileData = {
        file_name: file.name,
        file_type: file.type,
        total_chunks: totalChunks,
        file_chunks_hashs: chunkHashes, // Set chunk hashes here
        validate_hash: { hash: validateHash }
    };


    switch(type){
        case FileUploadType.V1:
            await v1_upload_file(uploadData, fileChunks, onProgress);
            break;
        case FileUploadType.V2:
            await v2_upload_file(deflatedData, uploadData, onProgress);
            break;
        case FileUploadType.V3:
            await v3_upload_file(deflatedData, uploadData, onProgress);
            break;
        case FileUploadType.WS1:
            await ws1_upload_file(fileChunks, onProgress);
            break;
        case FileUploadType.WS2:
            await ws2_upload_file(file, fileChunks, onProgress);
            break;
    }


}




async function v1_upload_file(uploadData: DTOFileData, fileChunks:DTOFileChunk[], onProgress: (progress: FileUploadProgress) => void){
    const data = await upload_data(uploadData);
    match(data)
    //BoardEvents
    .with({ Successful: P.select() }, async (boardEvent) => {
        await new Promise(r => setTimeout(r, 1000));
        await processChunksWithLimit(boardEvent, fileChunks, uploadData.total_chunks, onProgress);
    })
    .with({ Failed: P.select() }, (error) => console.error(error))
    .exhaustive();
}

async function v2_upload_file(deflatedData: ArrayBuffer, uploadData: DTOFileData, onProgress: (progress: FileUploadProgress) => void){
    const data = await upload_data(uploadData);
    match(data)
    //BoardEvents
    .with({ Successful: P.select() }, async (boardEvent) => {
        await new Promise(r => setTimeout(r, 1000));
        await upload_chunk2(deflatedData, uploadData.file_name);
    })
    .with({ Failed: P.select() }, (error) => console.error(error))
    .exhaustive();
}

async function v3_upload_file(deflatedData: ArrayBuffer, uploadData: DTOFileData, onProgress: (progress: FileUploadProgress) => void){
    const form = new FormData();
    const blob2 = new Blob([deflatedData], { type: uploadData.file_type });
    form.append('file_data', blob2);

    const xhr = new XMLHttpRequest();
    
    xhr.open('POST', 'api/upload/filechunk3', true);
    xhr.setRequestHeader('Content-Type', 'multipart/form-data');
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
            const currentTime = performance.now();

            const speed = event.loaded / ((currentTime - startTime) / 1000);

            onProgress({ loaded: progress, total: 100, speed: speed.toString()});
        }
    }



    xhr.onerror = function() {
        console.error('Failed to upload chunk');
    }

    xhr.send(form);
}




async function ws1_upload_file(fileChunks:DTOFileChunk[], onProgress: (progress: FileUploadProgress) => void){
    let isOpen = false;
    const ws = new WebSocket('ws://localhost:8000/filews');
    ws.onopen = async function open() {
        isOpen = true;
    
        // Start time measurement
        const startTime = performance.now();
    
        let totalBytesSent = 0;
        const totalChunks = fileChunks.length;
    
        for (let i = 0; i < totalChunks; i++) {
            if (!isOpen) {
                console.error('Connection is closed');
                break;
            }
    
            let chunkArray = fileChunks[i].chunk;
    
            // Ensure the chunk size is exactly CHUNK_SIZE
            if (chunkArray.byteLength > CHUNK_SIZE) {
                chunkArray = chunkArray.slice(0, CHUNK_SIZE);
            } else if (chunkArray.byteLength < CHUNK_SIZE) {
                // Handle last chunk or incomplete chunks
            }
    
            totalBytesSent += chunkArray.byteLength;
    
            ws.send(chunkArray);
    
            // Throttle sending rate (uncomment if needed)
            // await new Promise(r => setTimeout(r, 10));
    
            // Calculate and log speed every chunk
            const currentTime = performance.now();
            const elapsedTime = (currentTime - startTime) / 1000; // Convert ms to seconds
            const speed = formatSpeed(totalBytesSent, elapsedTime);
            onProgress({ loaded: i + 1, total: totalChunks, speed });
        }
    
        // Final speed after all chunks have been sent
        const endTime = performance.now();
        const totalTime = (endTime - startTime) / 1000; // Convert ms to seconds
        const finalSpeed = formatSpeed(totalBytesSent, totalTime);
        console.log(`Final speed: ${finalSpeed}`);
        console.log("Time taken: ", totalTime, "seconds");
        console.log("Total bytes sent: ", totalBytesSent);
        ws.close();
    };
    
    ws.onclose = function close() { 
        isOpen = false;
        console.log('Disconnected');
    };
    
    ws.onerror = function error() {
        isOpen = false;
        console.error('Connection error');
    };
}

async function ws2_upload_file(file:File, fileChunks:DTOFileChunk[], onProgress: (progress: FileUploadProgress) => void){
    let isOpen = false;
    const ws = new WebSocket('ws://localhost:8000/filews');
    let totalBytesSent = 0;
    let chunkIndex = 0;
    const startTime = performance.now();
    const totalChunks = fileChunks.length;
    ws.onopen = function() {
        isOpen = true;
        sendChunk();
    };
    
    ws.onmessage = function(event) {
        if (isOpen === false) {
            console.error('Connection is closed');
            return;
        }
        if (event.data === 'ack') {
            chunkIndex++;
            if (chunkIndex * CHUNK_SIZE < file.size) {
                sendChunk();
            } else {
                console.log('File transfer complete');
            }
        }
    };


    function sendChunk() {
        if (isOpen === false) {
            console.error('Connection is closed');
            return;
        }
        const chunkArray = fileChunks[chunkIndex].chunk;
        

        totalBytesSent += chunkArray.byteLength;

    
        ws.send(chunkArray);

        // Throttle sending rate (uncomment if needed)
        // await new Promise(r => setTimeout(r, 10));

        // Calculate and log speed every chunk
        const currentTime = performance.now();
        const elapsedTime = (currentTime - startTime) / 1000; // Convert ms to seconds
        const speed = formatSpeed(totalBytesSent, elapsedTime);
        onProgress({ loaded: chunkIndex + 1, total: totalChunks, speed });
        if (chunkIndex === totalChunks - 1) {
            ws.close();
        }
    }



    ws.onclose = function close() { 
        isOpen = false;
        const endTime = performance.now();
        const totalTime = (endTime - startTime) / 1000; // Convert ms to seconds
        const finalSpeed = formatSpeed(totalBytesSent, totalTime);
        console.log(`Final speed: ${finalSpeed}`);
        console.log("Time taken: ", totalTime, "seconds");
        console.log("Total bytes sent: ", totalBytesSent);
        console.log('Disconnected');
    };
    
    ws.onerror = function error() {
        isOpen = false;
        console.error('Connection error');
    };
}


















async function processChunksWithLimit(token: DTOFileToken, chunks: DTOFileChunk[],total_chunks:number, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const failedChunks: DTOFileChunk[] = [];
    let progress = 0;
    let totalBytesSent = 0;
    const startTime = performance.now();



    const processChunk = async (chunk: DTOFileChunk, retries: number = 0): Promise<void> => {
        try {        
            await new Promise(r => setTimeout(r, 1000));
            const added = await upload_file_chunk(chunk, token);
            if(added){
                progress++;
                totalBytesSent += chunk.chunk.byteLength;
                const speed = formatSpeed(totalBytesSent, (performance.now() - startTime) / 1000);
                onProgress({ loaded: (progress- failedChunks.length), total: total_chunks , speed});
            }

        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        } catch (error) {
            if (retries < RETRY_LIMIT) {
                console.warn(`Retrying chunk ${chunk.index} (${retries + 1}/${RETRY_LIMIT})`);
                await processChunk(chunk, retries + 1);
            } else {
                console.error(`Failed to upload chunk ${chunk.index} after ${RETRY_LIMIT} retries`);
                failedChunks.push(chunk);
            }
        }
    };

    // Process chunks in batches with a maximum number of parallel uploads
    const chunksQueue = [...chunks];
    while (chunksQueue.length > 0) {
        const activeUploads = chunksQueue.splice(0, MAX_PARALLEL_UPLOADS).map(chunk => processChunk(chunk));
        await Promise.all(activeUploads);
    }

    // Retry failed chunks if necessary
    if (failedChunks.length > 0) {
        console.log('Retrying failed chunks');
        await processChunksWithLimit(token,failedChunks, total_chunks, onProgress);
    }
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


async function upload_file_chunk(chunk: DTOFileChunk, token:DTOFileToken): Promise<boolean> {
    try {
        const res = await upload_chunk(chunk.chunk, chunk.file_name, chunk.index, chunk.validate_hash.hash, token);
        if (!res.ok) {
            throw new Error(`Failed to upload chunk ${chunk.index}`);
        }
        return true;
    }
    catch (error) {
        console.error('Failed to upload chunk:', error);
        return false;
    }
}

export async function upload_file2(chunk:any, file_name:string): Promise<boolean> {
    try {
        const res = await upload_chunk2(chunk,file_name);
        if (!res.ok) {
            throw new Error(`Failed to upload chunk ${file_name}`);
        }
        return true;
    }
    catch (error) {
        console.error('Failed to upload chunk:', error);
        return false;
    }
}

function concatenateUint8Arrays(chunks: Uint8Array[]): Uint8Array {
    const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.length;
    }

    return result;
}


function formatSpeed(bytes, seconds) {
    if (seconds === 0) return '0 MB/s';
    const megabytes = bytes / (1024 * 1024);
    const speed = megabytes / seconds;
    return `${speed.toFixed(2)} MB/s`;
}