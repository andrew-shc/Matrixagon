use std::collections::VecDeque;
use crate::event::types::ChunkEvents;

pub mod types;

#[derive(Clone)]
pub struct EventQueue<T: EventType<T> + Copy + Clone> {
    queue: VecDeque<T>,
}

impl<T: EventType<T> + Copy + Clone> EventQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn merge_events(&mut self, event: Vec<T>) {
        for e in event {
            self.add_event(e);
        }
    }

    pub fn add_event(&mut self, event: T) {
        self.queue.push_back(event);
    }

    pub fn run_event(&mut self, mut exec: impl FnMut(T)) {
        // creates a separate queue when executing to prevent accidental infinite feedback loop
        let mut front = self.queue.clone();

        loop {
            if front.is_empty() {
                exec(T::final_event());
                break;
            } else {
                exec(front.pop_back().unwrap());
            }
        }
    }

    pub fn flush_events(&mut self) {
        self.queue.clear();
    }
}

pub trait EventType<T: EventType<T>> {
    fn final_event() -> T;  // returns the final event of that event type
}
