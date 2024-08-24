<script lang="ts">

	import type { DtoQuestion, NumberScope } from 'cult-common';
	import { onMount } from 'svelte';
	import { BlobType, downloadBlob, getBlobType, type FileDownloadProgress } from './blodUtils';
	import ImageBlob from './ImageBlob.svelte';
	import VideoBlob from './VideoBlob.svelte';
	import AudioBlob from './AudioBlob.svelte';
	import TextBlob from './TextBlob.svelte';
	import { match, P } from 'ts-pattern';
	import { CurrentSessionsStore } from '$lib/stores/SessionStore';
	import { CookieStore } from '$lib/stores/cookies';

	export let current: DtoQuestion;

	let blob: Blob | null = null;
	let fileDownloadProgress: FileDownloadProgress | null = null;
	let blobType: BlobType = BlobType.UNKNOWN;
	let ranges : NumberScope[] = []

	const onProgress = (progress: FileDownloadProgress) => {
        if (blob != undefined)  return;
		if (progress.blob) {
            blobType = getBlobType(progress.blob);
			blob = progress.blob;
		} else {
			fileDownloadProgress = progress;
		}
	};

	async function loadBlob() {
		if (!blob) {
            match(current.question_type)
            .with({ Video: P.select() }, async (vid) => {
				ranges = vid.range;
                  await downloadBlob(vid.name, onProgress);
            })
            .otherwise(() => {
                console.log("Unsupported file type");
            });
		}
	}

    function isAdmin(): boolean {
        return CurrentSessionsStore
            .getSessionById({ id: $CookieStore.userSessionId.id})
            .is_admin;
    }

	onMount(loadBlob);
</script>

{#if blob}
	{#if blobType === BlobType.IMAGE}
		<ImageBlob image={blob} />
	{:else if blobType === BlobType.VIDEO}
		<VideoBlob video={blob} ranges={ranges} currUserIsAdmin = {isAdmin()} />
	{:else if blobType === BlobType.AUDIO}
		<AudioBlob audio={blob} />
	{:else if blobType === BlobType.TEXT}
		<TextBlob text={blob} />
	{:else}
		<p>Unsupported file type</p>
	{/if}
{:else}
	<p>Loading... {fileDownloadProgress?.current || 0} % /  {100} %  | Speed {fileDownloadProgress?.speed} </p>
{/if}
