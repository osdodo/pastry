#[derive(Debug, Clone)]
pub enum Message {
    ClosePage,
    /// Deferred content loading - page opens first, then content renders
    DeferredLoad(String),
    ToggleFold(String),
    QueryChanged(String),
    QuerySubmitted,
    CopyText(String),
    SelectionStarted(usize, usize),
    SelectionUpdated(usize, usize),
    SelectionEnded,
    Tick,
    Scrolled(iced::widget::scrollable::Viewport),
    StartDrag,
}
