// Defines the visual glyphs used throughout the application
// CLI output must stay pure ASCII
// TUI output may use Unicode for a richer look

pub(crate) struct Glyphs {
    pub task_playing: &'static str,
    pub task_stopped: &'static str,
    pub horizontal_rule: &'static str,
    pub empty_slot: &'static str,
    pub cursor_block: &'static str,
    pub smartcard: &'static str,
    pub book: &'static str,
}

pub(crate) const CLI: Glyphs = Glyphs {
    task_playing: ">>",
    task_stopped: "[]",
    horizontal_rule: "--",
    empty_slot: "-",
    cursor_block: "_",
    smartcard: "[card]",
    book: "[book]",
};

pub(crate) const TUI: Glyphs = Glyphs {
    task_playing: "\u{25B6}",            // ▶
    task_stopped: "\u{23F9}",            // ⏹
    horizontal_rule: "\u{2500}\u{2500}", // ──
    empty_slot: "\u{2014}",              // —
    cursor_block: "\u{2588}",            // █
    smartcard: "\u{1F511}",              // 🔑
    book: "\u{1F56E}",                   // 🕮
};
