import type { MediaState } from "cult-common";
import type { Snippet } from "svelte";


export enum FileUploadType {
    BOARDJSON,
    MEDIA,
}


export enum QuestionTypes  {
    MEDIA,
    YOUTUBE,
    QUESTION,
    NONE,
}



export enum VideoPlayerType {
    NONE,
    VIDEO,
    YOUTUBE,
}

export type BoardContext = {
  requestPlay: () => boolean;
  requestPause: (value: number) => boolean;
  changeMediaState: (state: MediaState) => void;
  requestSyncBackward: () => void;
  requestSyncForward: (calculated_diff: number) => void;
};

export type MediaPlayerContext = {
    changeState(state: MediaState): void;    
}

export type ButtonProps = {
    onclick: () => void;
    text?: string;
}

export interface BaseButtonProps extends ButtonProps {
    Icon: Snippet;
}