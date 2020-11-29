use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;
use std::mem;
use std::any::{TypeId, Any};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;

pub mod types;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EventName(pub &'static str);

// link: https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6b6fb317914af0be5545e90fe17fc472
// A struct used for transmitting event data to the callback closures
// The data is expected to be shared using Rc<> pointer, and are not mutably changed.
#[derive(Clone, Debug)]
pub struct EventData {
    // just to compartmentalize code between adding data before emitting and using the data while emitting
    packed: bool,
    // to signify this EventData is to be released to the closures
    released: bool,
    data: Rc<RefCell<VecDeque<Box<dyn Any>>>>,
    cycled: u8,  // an internal tracking number to track how many times the values was popped
    type_info: Vec<TypeId>,
}

impl EventData {
    pub fn new() -> Self {
        Self {
            packed: false,
            released: false,
            data: Rc::new(RefCell::new(VecDeque::new())),
            cycled: 0,
            type_info: Vec::new(),
        }
    }

    // for literals, writing out the type is really important
    // u32 vs u64 is really a big difference
    pub fn push<T: Clone + 'static>(&mut self, param: T) {
        if !self.packed {
            (*self.data).borrow_mut().push_back(Box::new(param) as Box<dyn Any>);
            self.type_info.push(TypeId::of::<T>());
        } else {
            panic!("Pushing new value after `pack()`");
        }
    }

    // packs the value just makes it so there are only one area where you can push
    // and another compartmentalized area to pop these values
    // and also this produces type info that will be used later in EventDispatcher
    fn pack(&mut self) {
        self.packed = true;
    }

    // used when the EventDispatcher calls clear on events, doesn't
    // raise the EventData with error at Drop
    fn release(&mut self) {
        self.released = true;
    }

    fn type_info(&self) -> Vec<TypeId> {
        if self.packed {
            // the type info is assumed to always be Some(T) after `pack()`
            let typ = self.type_info.clone();
            // to make sure numbers of types is exact to the amount of the actual data
            assert_eq!(typ.len(), (*self.data).borrow_mut().len());
            typ
        } else {
            panic!("Retrieving type information of an Event Data without first calling `pack()`");
        }
    }

    pub fn pop<T: Clone + 'static>(&mut self) -> T {
        if self.packed && self.released {
            if self.cycled > self.type_info.len() as u8 {
                self.cycled = self.type_info.len() as u8;  // to satisfy the panic on Drop to avoid confusion due to cycles "cycling" again
                panic!(format!("Too many values popped! There are only <{:?}> values within this EventData", self.type_info.len()));
            }

            let dt = (*self.data).borrow_mut().pop_front().expect("No more values");
            let param = *dt.downcast::<T>().expect("Invalid Type Parameter");

            (*self.data).borrow_mut().push_back(Box::new(param.clone()) as Box<dyn Any>);
            self.cycled += 1;
            param
        } else {
            panic!("Retrieving parameter values without first calling `pack()` or `release()`");
        }
    }
}

impl Drop for EventData {
    fn drop(&mut self) {
        if self.released {
            if self.cycled < self.type_info.len() as u8 {
                panic!(format!("There are unused values in this EventData! <{:?}> values still went unused! There are <{:?}> elements in this EventData", self.type_info.len() as u8-self.cycled, self.type_info.len()));
            }
        }
    }
}

// easily creates a Vec-based EventData
#[macro_export]
macro_rules! event_data {
    [$($val:expr),* $(,)?] => {
        {
            let mut evd = crate::event::EventData::new();
            $(evd.push($val);)*
            evd
        }
    }
}

pub struct EventDispatcher {
    // a map of registered events with each of its respective type information
    event_names: HashMap<EventName, Vec<TypeId>>,

    // // a list of all emitted events since instanced
    // record: Vec<(EventName, EventData)>,

    // a synchronized way to dispatch all the emitted events at the same time
    events_buf: RefCell<Vec<(EventName, EventData)>>,
    // a list of all emitted events since last flushed
    events: RefCell<Vec<(EventName, EventData)>>,
}

impl EventDispatcher {
    pub fn new(event_names: HashMap<EventName, Vec<TypeId>>) -> Rc<Self> {
        Rc::new(Self {
            event_names: event_names,
            // record: Vec::new(),
            events_buf: RefCell::new(Vec::new()),
            events: RefCell::new(Vec::new()),
        })
    }

    pub fn emit(self: Rc<Self>, name: EventName, mut data: EventData) {
        data.pack();
        // self.record.push((name, data.clone()));
        if self.event_names.keys().collect::<Vec<_>>().contains(&&name) {
            if data.type_info() == self.event_names[&name] {
                (*self.events_buf.borrow_mut()).push((name, data));
            } else {
                panic!(format!("[EVENT:EMIT] EventName <{:?}> emitted data with invalid types", name));
            }
        } else {
            panic!(format!("[EVENT:EMIT] EventName <{:?}> is not a registered event within this EventDispatcher", name));
        }
    }

    // to receive and call the closure for all the selected events since last flushed
    pub fn receive<T: FnMut(EventData)>(self: Rc<Self>, event: EventName, mut closure: T) {
        if self.event_names.keys().collect::<Vec<_>>().contains(&&event) {
            for (name, data) in &*self.events.borrow_mut() {
                if name == &event {
                    let mut param = data.clone();
                    param.release();
                    closure(param);
                }
            }
        } else {
            panic!(format!("[EVENT:RECEIVE] EventName <{:?}> is not a registered event within this EventDispatcher", event));
        }
    }

    // to receive and call the closure once of the first selected event since last flushed
    pub fn receive_once<T: FnMut(EventData)>(self: Rc<Self>, event: EventName, mut closure: T) {
        if self.event_names.keys().collect::<Vec<_>>().contains(&&event) {
            for (name, data) in &*self.events.borrow_mut() {
                if name == &event {
                    closure(data.clone());
                    break;
                }
            }
        } else {
            panic!(format!("[EVENT:RECEIVE_ONCE] EventName <{:?}> is not a registered event within this EventDispatcher", event));
        }
    }

    // clears the current running events (self.events) and replaces it with new events from previous "iteration" (self.events_buf)
    pub fn event_swap(self: Rc<Self>) {
        // println!("Event Buffer    : {:?}", *self.events_buf.borrow_mut());
        // println!("Event Presenting: {:?}", *self.events.borrow_mut());
        (*self.events.borrow_mut()).clear();
        (*self.events.borrow_mut()).append(&mut *self.events_buf.borrow_mut());
    }
}
