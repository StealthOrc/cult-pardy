import { inflate, deflate } from 'fflate';
import { XXH64 } from 'xxh3-ts';
import { Buffer } from 'buffer';
import type { DTOFileChunk, DTOFileData, DTOFileToken, FileChunkHash, FileDataForm, FileDataReponse } from 'cult-common';
import { upload_chunk, upload_chunk2, upload_chunk3, upload_data } from '$lib/api/ApiRequests';
import { match, P } from 'ts-pattern';  
import axios from 'axios';


const CHUNK_SIZE = 25_000; // 200 KB 
const MAX_PARALLEL_UPLOADS = 5; // Maximum number of parallel uploads
const RETRY_LIMIT = 3; // Number of retry attempts for failed chunks


export type FileUploadProgress = {
    loaded: number;
    total: number;
    speed: string;
};

export async function handleFileUpload(file: File, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
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
    const startTime = performance.now();
    let totalBytesSent = 0;
    function formatSpeed(bytes, seconds) {
        if (seconds === 0) return '0 MB/s';
        const megabytes = bytes / (1024 * 1024);
        const speed = megabytes / seconds;
        return `${speed.toFixed(2)} MB/s`;
    }


    /*const ws = new WebSocket('ws://localhost:8000/filews');
    console.log(fileChunks.length);

    function formatSpeed(bytes, seconds) {
        if (seconds === 0) return '0 MB/s';
        const megabytes = bytes / (1024 * 1024);
        const speed = megabytes / seconds;
        return `${speed.toFixed(2)} MB/s`;
    }

    let isOpen = false;
    const startTime = performance.now();
    let totalBytesSent = 0;
    let chunkIndex = 0;*/

    const form = new FormData();
    let blob2 = new Blob([deflatedData], { type: file.type });
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

    xhr.upload.onprogress = function(event) {
        if (event.lengthComputable) {
            const currentTime = performance.now();
            const elapsedTime = (currentTime - startTime) / 1000; // Convert ms to seconds
            const speed = formatSpeed(event.loaded, elapsedTime);
            onProgress({ loaded: event.loaded, total: totalChunks, speed });
        }
    }



    xhr.onerror = function() {
        console.error('Failed to upload chunk');
    }

    xhr.send(form);




    /*ws.onopen = function() {
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




    */


    /*
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

    
    return;

    const CHUNK_SIZE2 = 1024




    match(data)
    //BoardEvents
    .with({ Successful: P.select() }, async (boardEvent) => {
        await new Promise(r => setTimeout(r, 1000));
       // const res = await upload_file2(deflatedData, file.name);
        const form = new FormData();
        const blob = new Blob([deflatedData], { type: file.type });
        /*try {
            const xhr = new XMLHttpRequest();
            form.append('file_data', blob);
    
    
            xhr.open('POST', 'http://localhost:8000/api/upload/filechunk3', true);
            xhr.setRequestHeader('Content-Type', 'multipart/form-data');
            xhr.onerror = function() {
                console.error('Failed to upload chunk');
            }

            xhr.send(form);
        
    
        } catch (error) {
            console.error('Failed to upload chunk:', error);
        }





        //form.append('file_data', blob);
        


        //await upload_chunk3(form);

        //await processChunksWithLimit(boardEvent, fileChunks, uploadData.total_chunks, onProgress);
    })
    .with({ Failed: P.select() }, (error) => console.error(error))
    .exhaustive();
    */

}

async function processChunksWithLimit(token: DTOFileToken, chunks: DTOFileChunk[],total_chunks:number, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const failedChunks: DTOFileChunk[] = [];
    let progress = 0;

    const processChunk = async (chunk: DTOFileChunk, retries: number = 0): Promise<void> => {
        try {        
            await new Promise(r => setTimeout(r, 1000));
            const added = await upload_file_chunk(chunk, token);
            if(added){
                progress++;
                onProgress({ loaded: (progress- failedChunks.length), total: total_chunks , speed: '0 MB/s' });
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

