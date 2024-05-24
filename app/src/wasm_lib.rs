use gloo_console::log;
use wasm_bindgen::prelude::*;

//src: https://developers.google.com/youtube/player_parameters
#[wasm_bindgen(module = "/js/youtube.js")]
extern "C" {
    /// creates a YT.Player object
    pub fn createYTPlayer(videoId: String);
}
