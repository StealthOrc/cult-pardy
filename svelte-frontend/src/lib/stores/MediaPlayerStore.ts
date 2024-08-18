import type { MediaPlayerContext } from "$lib/types";
import { readable, writable } from "svelte/store";

export const mediaPlayerStore = writable<MediaPlayerContext | null>(null)