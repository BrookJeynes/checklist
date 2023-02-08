pub struct Task {
    pub completed: bool,
    pub content: String,
}

impl Task {
    pub fn select(&mut self) {
        self.completed = !self.completed;
    }
}
