use std::collections::VecDeque;

#[derive(Debug)]
pub struct TaskQueue<T> {
    pub tasks: VecDeque<T>,
}

impl<T> TaskQueue<T> {
    pub fn new() -> Self {
        TaskQueue {
            tasks: VecDeque::new()
        }
    }

    pub fn pushb(&mut self, val: T) {
        self.tasks.push_back(val);
    }

    pub fn popf(&mut self) -> Option<T> {
        self.tasks.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn peek(&self) -> Option<&T> {
        self.tasks.front()
    }
}
