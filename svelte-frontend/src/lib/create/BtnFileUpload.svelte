<script lang="ts">
	import { upload_chunk, upload_data } from "$lib/api/ApiRequests";
	import { fileToBinary } from "$lib/BinaryConversion";
	import type { ApiResponse, DTOFileChunk, DTOFileData, FileChunk, ValidateHash } from "cult-common";
	import { deflateSync } from "fflate";
    import { XXH64 } from 'xxh3-ts';
    import { Buffer } from 'buffer';


    async function changed(event: Event) {
        var input = event.target as HTMLInputElement;
        console.log(input.files);
        if (!input.files) return;
        var file = input.files[0];
        if (file) {
            fileToBinary(file, async (data) => {
                console.log("File Data");
                console.log(data);
                console.log("Deflating");
                console.log(new Uint8Array(data));
                console.log("Size before: " + data.byteLength);
                let test = deflateSync(new Uint8Array(data)).buffer;
                console.log("Size after: " + test.byteLength);


                let u8 = new Uint8Array(data);
                let validate_hash = XXH64(Buffer.from(u8.buffer,0)).toString();
                let max_size = 200_000;
                
                let number = Math.ceil(file.size / max_size);



                let hashes :ValidateHash[] = [];
                let requests : DTOFileChunk[] = [];
                for (let i = 0; i < number; i++) {
                    let start = i * max_size;
                    let end = Math.min((i + 1) * max_size, file.size);
                    let chunkArray = new Uint8Array(data.slice(start, end));
                    let hash =  {
                            hash: XXH64(Buffer.from(chunkArray.buffer,0)).toString()
                        };

                    let chunk :DTOFileChunk =  {
                        file_name: file.name,
                        index: i,
                        chunk: Array.from(chunkArray),
                        validate_hash: hash
                    }
                    hashes.push(hash);
                    requests.push(chunk);
                }

                let uploadData: DTOFileData = {
                    file_name: file.name,
                    file_type: file.type,
                    total_chunks: number,
                    file_chunks_hashs: hashes,
                    validate_hash: {
                        hash: validate_hash
                    },
                }
                
                console.log("Uploading Data");




                upload_data(uploadData).then((response) => {
                    console.log("Data Response" + response);
                    //promise all chunks with upload_chunk
                    let chunks : Promise<ApiResponse>[]= [];
                    requests.forEach((chunk) => {
                        chunks.push(upload_chunk(chunk));
                    });
                    Promise.all(chunks).then((responses) => {
                        console.log("Chunk Responses" + responses);
                    }
                ).catch((error) => {
                    console.log(error);
                });


                }).catch((error) => {
                    console.log(error);
                });

        
            });
        }

    }
</script>

<input class="rounded bg-gray-700 p-2" type="file" id="file" accept="image/*, video/*" on:change={changed} />