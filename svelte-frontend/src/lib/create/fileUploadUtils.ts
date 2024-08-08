import { Deflate,} from 'fflate';
import { XXH64 } from 'xxh3-ts';
import { Buffer } from 'buffer';
import type { DTOFileChunk, DTOFileData, DTOFileToken, FileChunkHash, FileDataReponse } from 'cult-common';
import { upload_chunk, upload_data } from '$lib/api/ApiRequests';
import { match, P } from 'ts-pattern';

const CHUNK_SIZE = 200_000; // 200 KB
const MAX_PARALLEL_UPLOADS = 5; // Maximum number of parallel uploads
const RETRY_LIMIT = 3; // Number of retry attempts for failed chunks


export type FileUploadProgress = {
    loaded: number;
    total: number;
};

export async function handleFileUpload(file: File, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // Compress the file data asynchronously
    const deflatedData = await compressData(uint8Array);

    // Calculate hash of the deflated data
    const validateHash = XXH64(Buffer.from(deflatedData)).toString();
    const totalChunks = Math.ceil(deflatedData.byteLength / CHUNK_SIZE);
    onProgress({ loaded: 0, total: totalChunks });

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
            chunk: Array.from(chunkArray),
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

    // Upload metadata
    const data : FileDataReponse = await upload_data(uploadData);

    match(data)
    //BoardEvents
    .with({ Successful: P.select() }, async (boardEvent) => {
        await processChunksWithLimit(boardEvent, fileChunks, uploadData.total_chunks, onProgress);
    })
    .with({ Failed: P.select() }, (error) => console.error(error))
    .exhaustive();


}

async function processChunksWithLimit(token: DTOFileToken, chunks: DTOFileChunk[],total_chunks:number, onProgress: (progress: FileUploadProgress) => void): Promise<void> {
    const failedChunks: DTOFileChunk[] = [];
    let progress = 0;

    const processChunk = async (chunk: DTOFileChunk, retries: number = 0): Promise<void> => {
        try {
            const added = await upload_file_chunk(chunk, token);
            if(added){
                progress++;
                onProgress({ loaded: (progress- failedChunks.length), total: total_chunks });
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

async function compressData(data: Uint8Array): Promise<Uint8Array> {
    return new Promise((resolve, reject) => {
        try {
            const compressedChunks: Uint8Array[] = [];
            const deflate = new Deflate((u8array, final) => {
                compressedChunks.push(u8array);
                if (final) {
                    resolve(concatenateUint8Arrays(compressedChunks));
                }

            });
            deflate.push(data, true);
        } catch (error) {
            reject(error);
        }
    });
}



async function upload_file_chunk(chunk: DTOFileChunk, token:DTOFileToken): Promise<boolean> {
    try {
        const res = await upload_chunk(chunk, token);
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