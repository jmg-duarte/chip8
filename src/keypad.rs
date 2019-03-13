pub struct Keypad {
    pub keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad { keys: [false; 16] }
    }

    pub fn key_down(&mut self, index: usize) {
        self.keys[index] = true;
    }

    pub fn key_up(&mut self, index: usize) {
        self.keys[index] = false;
    }

    pub fn is_key_down(&self, index: usize) -> bool {
        self.keys[index]
    }
}
