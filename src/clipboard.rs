use arboard::Clipboard as Arboard;

/// Long-lived clipboard handle. `None` means the platform clipboard is not
/// reachable (e.g. headless Linux without X11/Wayland) — callers must show a
/// message instead of crashing.
pub struct Clipboard {
    inner: Arboard,
}

impl Clipboard {
    pub fn new() -> Option<Self> {
        Arboard::new().ok().map(|inner| Clipboard { inner })
    }

    pub fn set_text(&mut self, text: &str) -> Result<(), arboard::Error> {
        self.inner.set_text(text.to_string())
    }
}
