<script lang="ts">
    import { base } from "$app/paths";
	import { onMount } from "svelte";
    import { CookieStore, dev_loaded,type SessionCookies } from "$lib/stores/cookies";
	import { type DiscordUser } from "cult-common";
	import PlayerIcon from "./PlayerIcon.svelte";
	import { discord_session} from "$lib/api/ApiRequests";
	import FileUpload from "$lib/create/FileUpload.svelte";
	import { FileUploadType } from "$lib/types";
    let lobbyid = '';

    let cookies : SessionCookies; 
    CookieStore.subscribe(value => {
            cookies = value;
    });


  
    let discord_user: DiscordUser | null = null;
    let loaded = false;

    onMount(async () => {



        
        let session_res= await discord_session();
        if (session_res) {
            discord_user = session_res;
        }
        loaded = true;
    })



    function BinarytoaArrayBuffer(binary: string): ArrayBuffer {
        var binary_string = window.atob(binary);
        var len = binary_string.length;
        var bytes = new Uint8Array(len);    
        for (var i = 0; i < len; i++) {
            bytes[i] = Number(binary_string.at(i));
        }
        return bytes.buffer;
    }


    function arrayBufferToBinary(buffer: ArrayBuffer): string {
        //use window.btoa to convert binary to base64 for data URI
        var binary = '';
        var bytes = new Uint8Array(buffer);
        var len = bytes.byteLength;
        for (var i = 0; i < len; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        return window.btoa(binary);
    }

      function fileToBinary(file:File, callback: (binary: string) => void) {  
        var reader: FileReader = new FileReader();
        reader.onload = (event) => {
            callback(arrayBufferToBinary(reader.result as ArrayBuffer));
        };
        reader.readAsArrayBuffer(file);
      }

      var input = document.getElementById("file");
      var output = document.getElementsByClassName("binary");


    function changed(event: Event) {
        var input = event.target as HTMLInputElement;
        console.log(input.files);
        if (!input.files) return;
        var file = input.files[0];
        if (file) fileToBinary(file, (binary) => {
            console.log(binary);
            if (output == null) return;
            output[0].textContent = binary
            }
        );
    }

</script>



<div class=" h-dvh w-dvw flex flex-col items-center justify-center gap-2">
        {#if loaded}
        <input bind:value={lobbyid} class="border-2 border-blue-600 placeholder-blue-600 p-2 rounded m-2" type="text" name="lobby-id" id="lobby-id" placeholder="Lobby ID"/>
        <button on:click={() => window.location.href = `${base}/game/${lobbyid}`} class="bg-blue-600 text-white font-bold py-2 px-4 rounded focus:outline-none focus:ring-2 transition-all duration-200">Join Game</button>
        <button on:click={() => window.location.href = `${base}/game/main`} class="bg-blue-600 text-white font-bold py-2 px-4 rounded focus:outline-none focus:ring-2 transition-all duration-200">Join /main/ Game</button>
        <PlayerIcon {discord_user}/>
                {#if !discord_user}
                    <a id="discord_login" href="discord" class="flex flex-row bg-discord-blue focus:outline-none focus:ring-2 transition-all duration-200 text-white font-bold py-2 px-4 rounded-lg">
                        <svg class="mr-2 self-center" aria-hidden="true" role="img" width="30" height="30" fill="none" viewBox="0 0 24 24"><path fill="currentColor" d="M19.73 4.87a18.2 18.2 0 0 0-4.6-1.44c-.21.4-.4.8-.58 1.21-1.69-.25-3.4-.25-5.1 0-.18-.41-.37-.82-.59-1.2-1.6.27-3.14.75-4.6 1.43A19.04 19.04 0 0 0 .96 17.7a18.43 18.43 0 0 0 5.63 2.87c.46-.62.86-1.28 1.2-1.98-.65-.25-1.29-.55-1.9-.92.17-.12.32-.24.47-.37 3.58 1.7 7.7 1.7 11.28 0l.46.37c-.6.36-1.25.67-1.9.92.35.7.75 1.35 1.2 1.98 2.03-.63 3.94-1.6 5.64-2.87.47-4.87-.78-9.09-3.3-12.83ZM8.3 15.12c-1.1 0-2-1.02-2-2.27 0-1.24.88-2.26 2-2.26s2.02 1.02 2 2.26c0 1.25-.89 2.27-2 2.27Zm7.4 0c-1.1 0-2-1.02-2-2.27 0-1.24.88-2.26 2-2.26s2.02 1.02 2 2.26c0 1.25-.88 2.27-2 2.27Z"></path></svg>
                        <p class="self-center">Login with Discord</p>
                    </a>
                {/if}
        {/if}
        <div class="flex space-x-4">
            <FileUpload title="Upload Question File"/>
            <FileUpload title="Upload Board Json" uploadType={FileUploadType.BOARDJSON}/>
        </div>
</div>