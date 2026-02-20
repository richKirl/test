pub struct Stack<T> {
    stash: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        //чтобы не было аллокаций в рантайме
        Self {
            stash: Vec::with_capacity(64),
        }
    }

    pub fn push(&mut self, val: T) {
        self.stash.push(val);
    }

    // Возвращаем Option, так как в стеке может ничего не быть
    pub fn pop(&mut self) -> Option<T> {
        self.stash.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.stash.is_empty()
    }

    pub fn clear(&mut self) {
        self.stash.clear();
    }
}
