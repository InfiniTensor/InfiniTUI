static SPINNER_CHARS: &[char] = &['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'];

#[derive(Default, Debug, Clone)]
pub struct Spinner {
    pub active: bool,
    pub index: usize,
}

impl Spinner {
    pub fn draw(&self) -> char {
        SPINNER_CHARS[self.index]
    }

    pub fn update(&mut self) {
        self.index = (self.index + 1) % SPINNER_CHARS.len();
    }
}
