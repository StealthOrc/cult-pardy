function onPlayerStateChange(event) {
  console.log("onPlayerStateChange");
}

function onPlayerReady(event) {
  console.log("onPlayerReady");
}

export function createYTPlayer(videoId) {
  var tag = document.createElement("script");
  tag.id = "iframe-demo";
  tag.src = "https://www.youtube.com/player_api";
  //add script to head
  var parentNode = document.head;
  parentNode.appendChild(tag);
  window.onPlayerReady = onPlayerReady;
  window.videoId = videoId;
}

var player;
window.onYouTubePlayerAPIReady = function onYouTubePlayerAPIReady() {
  console.log("onYouTubeIframeAPIReady1 videoId:", window.videoId);
  //todo: somehow we need to make the title not show on the video
  //see this website for example: https://www.ted.com/talks/david_hooker_the_importance_of_visual_literacy
  player = new YT.Player("player", {
    videoId: window.videoId,
    width: "100%",
    height: "100%",
    playerVars: {
      enablejsapi: 1,
      showInfo: 0,
      controls: 1,
      disablekb: 1,
      iv_load_policy: 3,
      start: 10,
      fs: 1,
      autohide: 1,
      // origin: "http://localhost:8000",
    },
    events: {
      onReady: window.onPlayerReady,
      onStateChange: onPlayerStateChange,
    },
  });
  console.log("onYouTubeIframeAPIReady2");
};
