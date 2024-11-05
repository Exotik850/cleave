#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum SelectionMode {
    #[default]
    Move, // Move the selection
    InverseResize, // Make the selection smaller
    Resize,        // Make the selection larger
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
