
export function binaryToByteArray(binaryString: string): Uint8Array {
    if (binaryString.length % 8 !== 0) {
        throw new Error('Binary string length must be a multiple of 8');
    }
    
    const byteArray = new Uint8Array(binaryString.length / 8);
    
    for (let i = 0; i < byteArray.length; i++) {
        byteArray[i] = parseInt(binaryString.slice(i * 8, (i + 1) * 8), 2);
    }
    
    return byteArray;
}
export function arrayBufferToBinary(arrayBuffer: ArrayBuffer): string {
    const byteArray = new Uint8Array(arrayBuffer);
    return Array.from(byteArray)
        .map(byte => byte.toString(2).padStart(8, '0'))
        .join('');
}

export function fileToBinary(file:File, callback:(value: ArrayBuffer)=>void): void{  
    const reader: FileReader = new FileReader();
    console.log(file.size);
    reader.onload = async function() {
        callback(reader.result as ArrayBuffer);
    }
    reader.readAsArrayBuffer(file);
}
/**
 * Concatenates an array of Uint8Arrays into a single Uint8Array.
 *
 * @param {Uint8Array[]} chunks - The array of Uint8Arrays to concatenate.
 * @return {Uint8Array} - The concatenated Uint8Array.
 */
export function buildUint8ArrayFromChunks(chunks: Uint8Array[]): Uint8Array {
    const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.length;
    }

    return result;
}