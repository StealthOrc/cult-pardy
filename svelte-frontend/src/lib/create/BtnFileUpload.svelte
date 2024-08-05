<script lang="ts">
	import { upload } from "$lib/api/ApiRequests";
	import { fileToBinary } from "$lib/BinaryConversion";
	import type { myDTOFileData } from "cult-common";
	import { deflateSync } from "fflate";

    async function changed(event: Event) {
        var input = event.target as HTMLInputElement;
        console.log(input.files);
        if (!input.files) return;
        var file = input.files[0];
        if (file) {
            fileToBinary(file, async (data) => {
                let uploadData: myDTOFileData = {
                    image: deflateSync(new Uint8Array(data)),
                    file_name: file.name,
                    file_type: file.type
                }
                console.log(uploadData);
                await upload(uploadData);
            });
        }
    }
</script>

<input class="rounded bg-gray-700 p-2" type="file" id="file" accept="image/*, video/*" on:change={changed} />