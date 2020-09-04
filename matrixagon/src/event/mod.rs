use std::collections::vec_deque::VecDeque;


pub mod types;


// a global-local event for handling events for multiple wide components
#[derive(Clone)]
pub struct EventQueue<T: EventType<T> + Copy + Clone> {
    queue: VecDeque<T>,
    flow: ControlFlow,
}

impl<T: EventType<T> + Copy + Clone> EventQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            flow: ControlFlow::Next,
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

    // adds an important event that must immediately stop the event and run this particular event
    // note: do not abuse it
    // TODO: Waiting to be implemented
    pub fn add_event_priority(&mut self, event: T) {

    }

    pub fn run_event(&mut self, mut exec: impl FnMut(&T, &mut ControlFlow)) {
        self.flow = ControlFlow::Next;
        // creates a separate queue when executing to prevent accidental infinite feedback loop
        let mut front = self.queue.clone();

        loop {
            if let ControlFlow::Next = self.flow {
                if front.is_empty() {
                    exec(&T::final_event(), &mut self.flow);
                    break;
                } else {
                    exec(&front.pop_back().unwrap(), &mut self.flow);
                }
            } else if let ControlFlow::Halt = self.flow {
                exec(&T::final_event(), &mut self.flow);
                break;
            }
        }
    }

    pub fn flush_events(&mut self) {
        self.queue.clear();
    }

    // counts how many events are still awaiting to be consumed
    pub fn event_count(&self) -> usize {
        self.queue.len()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ControlFlow {
    Next,  // goes to next events, or end
    // Pause, ?
    Halt,  // ends the event with the final event
}

// registers event to be able to use EventQueue
pub trait EventType<T: EventType<T>> {
    fn final_event() -> T;  // returns the final event of that event type
}

// transfers the current events to another stack
pub trait EventTransfer<T: EventType<T>> {
    fn transfer_into(&mut self) -> Vec<T>;  // implements an event transfer from this event to another event stack
    fn transfer_copy(&self) -> Vec<T>;  // implements a copy of this event and transfer it to another event stack
    fn transfer_except(&mut self) -> Vec<T>;  // moves this events that are necessary to another event stack
}
