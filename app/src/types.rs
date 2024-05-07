use cult_common::Vector2D;

// Message for Yew App
#[derive(Clone, Copy)]
pub enum AppMsg {
    GetButtonQuestion(Vector2D),
    BoardUnloaded,
    BoardLoaded,
}
