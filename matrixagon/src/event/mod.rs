use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;
use std::mem;
use std::any::{TypeId, Any};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;

pub mod types;


// // TODO: Somehow to allow the emitting of a event be controlled, and be async
// // TODO: Also somehow allow super prioritize events to be run first, suspending and other operations
// // note: Event cannot be on a separate thread to maintain a FnMut
//
// // an event system for observer patterns: subscribe and emit events
// #[derive(Clone)]
// pub struct EventObserver<'a, 'ext, T: EventType<T> + Copy + Clone> {
//     // records all the emitted events before it gets cleared for the next frame
//     events: Vec<T>,
//     // a list of observers (observer name, observing event, closure)
//     observers: Vec<(String, EventName<'a>, Box<dyn FnMut(T) -> ()>)>,
//     // a vectors of other event queue to be connected with this event queue
//     external: Vec<&'ext Self>,
// }
//
// impl<'a, 'ext, T: EventType<T> + Copy + Clone> EventObserver<'a, 'ext, T> {
//     // subscribe: use this when you want to execute it immediately (observing) after that targetted event has been emitted
//     // on_notify: use this when you want to wait till the components are ready to check if the event has been emitted
//
//     // defines a new subject, observer struct
//     pub fn new() -> Self {
//         Self {
//             events: Vec::new(),
//             observers: Vec::new(),
//             external: Vec::new(),
//         }
//     }
//
//     // emits a new event to the event bus
//     pub fn emit(&self, event: T) {
//         // TODO: add events to itself and all the externals
//         for mut ext in self.external {
//             ext.emit(event);
//         }
//         // TODO: improvements: pre-generate the discrimants in the event's event_name() func
//         for (_name, _, f) in self.observers.iter()
//             .filter(|(_, e, _)| mem::discriminant(T::event_name(*e)) == mem::discriminant(event)) {
//             f(event);
//         }
//     }
//
//     // observes an events on sub-components initialization
//     // the subscriber will execute the closure when the specific event has been emitted
//     pub fn subscribe(&mut self, name: String, event: EventName<'a>, f: impl FnMut(T) -> ()) {
//         self.observers.push((name, event, Box::new(f)));
//     }
//
//     // unsubscribes the event using the name to find it
//     // this function returns false when the function can't find the observer's name
//     pub fn unsubscribe(&mut self, name: String) -> bool {
//         if self.observers.iter().any(|(nm, _, _)| nm == name) {
//             self.observers.retain(|(nm, _, _)| nm != name);
//             true
//         } else {
//             false
//         }
//     }
//
//     // this will connect the events from the parent queue to the child
//     // note: the connect only works one way, so to connect events from child to parent you have to create
//     //       call the connect() function separately
//     pub fn connect(&mut self, event_queue: &'ext Self) {
//         self.external.push(event_queue);
//     }
//
//     // this will return true, when called, if the event has been emitted
//     pub fn on_notify(&self, event: EventName<'a>) -> Option<T> {
//         for e in self.events.iter() {
//             if mem::discriminant(T::event_name(event)) == mem::discriminant(e) {
//                 return Some(*e);
//             }
//         }
//         None
//     }
//
//     // clears only the current event
//     pub fn clear_event(&mut self) {
//         self.events.clear();
//     }
// }
//
// // registers event to be able to use EventQueue
// pub trait EventType<T: EventType<T>> {
//     fn final_event() -> T;  // returns the final event of that event type
//     // TODO: AHHHHHHHHH, I dont like how **ENUM VARIANTS** are **NOT A TYPE**  >:(
//     // naming convention: use slashes to denote a level deeper in a event, and use standard rust enum variant names
//     fn event_name(test_ename: EventName<'static>) -> T;  // maps a list of event name (in strings) to a arbitrarily defined event enums
// }


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EventName(pub &'static str);

// an observer-based events to send out all relevant events to relevant observers stored in a list
// pub struct EventInterchange<'e, 'i, O>
//     where O: FnMut(dyn Any) -> () {
//     event_types: HashMap<EventName<'e>, TypeId>,
//     observers: Vec<(EventName<'e>, O)>,
//     connections: Vec<&'i EventInterchange<'i, 'i, O>>,
// }
//
// impl<'e, 'i, O> EventInterchange<'e, 'i, O> {
//     pub fn new(events: HashMap<EventName<'e>, TypeId>) -> Self {
//         Self {
//             event_types: events,
//             observers: Vec::new(),
//             connections: Vec::new(),
//         }
//     }
//
//     // emits a new events and send it to all of the observers stored in this interchange or connected ones
//     // event is the event name in strings coupled with data that is associated with the event
//     pub fn emit<T: Debug + Clone>(&self, event: EventName, data: T) {
//         if let Some(tid) = self.event_types.get(&event) {
//             if tid == TypeId::of::<T>() {
//                 for (obs_ename, closure) in self.observers {
//                     if event == obs_ename {
//                         // found observers with its event observing the same with the currently emitting event
//                         closure(data.clone());
//                     }
//                 }
//                 for conc in self.connections {
//                     conc.emit(event, data.clone());
//                 }
//             } else {
//                 println!("[EVENT EMIT] The event <Event: {:?}> emitting with the data has an invalid data type: {:?}", event, data.clone());
//             }
//         } else {
//             println!("[EVENT EMIT] Event not found: {:?}", event);
//         }
//     }
//
//     // adds a new closure observer that will be executed whenever the relevant event is emitted
//     // subscribes the closure parameter to the event
//     pub fn subscribe<T>(&mut self, event: EventName<'e>, closure: impl FnMut(T) -> ()) {
//         if let Some(tid) = self.event_types.get(&event) {
//             if tid == TypeId::of::<T>() {
//                 self.observers.push((event, closure));
//             } else {
//                 println!("[EVENT SUBSCRIBE] An observer subscribing to <Event: {:?}> has passed an invalid mutable closure parameter for the respecting event", event);
//             }
//         } else {
//             println!("[EVENT SUBSCRIBE] Event not found: {:?}", event);
//         }
//     }
//
//     // unsubscribes any observers that are subscribed to the selected event to be unsubscribed
//     pub fn unsubscribe(&mut self, event: EventName<'e>) {
//         self.observers.retain(|(ename, _)| ename != event);
//     }
//
//     // connects the other event interchange to this event interchange
//     pub fn connect(&mut self, other: &'i EventInterchange<'e, 'i, O>) {
//         self.connections.push(other);
//     }
// }

//
// pub struct EventDispatcher {
//     event_types: HashMap<EventName, TypeId>,
//     // a list of observers: (The event name it is listening to, the closure it calls when the event is emitted)
//     // TODO: How about a vectors of Vec<Box<dyn Any>> so FnMut(Vec<Box<dyn Any>>) -> ()
//     observers: RefCell<Vec<(EventName, Box<dyn Any>)>>,
// }
//
// impl EventDispatcher {
//     pub fn new(events: HashMap<EventName, TypeId>) -> Rc<Self> {
//         Rc::new(
//             Self {
//                 event_types: events,
//                 observers: RefCell::new(Vec::new()),
//             }
//         )
//     }
//
//     // emits a new events and send it to all of the observers stored in this interchange or connected ones
//     // event is the event name in strings coupled with data that is associated with the event
//     #[allow(mutable_transmutes)]
//     pub fn emit<T: Debug + Clone + 'static>(self: Rc<Self>, event: EventName, data: T) {
//         if let Some(tid) = self.event_types.get(&event) {
//             if tid == &TypeId::of::<T>() {
//                 for (obs_ename, ref mut closure) in &mut *self.observers.borrow_mut() {
//                     if &event == obs_ename {
//                         // found observers with its event observing the same with the currently emitting event
//                         // (closure.downcast::<dyn FnMut(T) -> ()>())
//
//                         println!("============================== A ==============================");
//                         // let func =  unsafe {
//                         //     mem::transmute::<&mut Box<dyn Any>, &mut Box<dyn FnMut(T) -> ()>>(closure)
//                         // };
//                         //
//                         // (**func)(data.clone());
//                         use crate::ui::Widget;
//                         // let foinc = closure.downcast_mut::<&mut dyn FnMut(T)>().expect("NO ERRORS");
//                         // let foinc = closure.downcast_mut::<&mut dyn FnMut(u32, u32)>().expect("NO ERRORS");
//
//                         println!("============================== A ==============================");
//
//                         if let Some(func) = (&mut *closure).downcast_mut::<Box<dyn FnMut(T)>>() {
//                             println!("______________________________ B ______________________________");
//                             // let f = **func;
//                             unsafe {
//                                 // let f = mem::transmute::<&Box<dyn FnMut(T)>, &mut Box<dyn FnMut(T)>>(func);
//                                 (**func)(data.clone());
//                             }
//                         } else {
//                             println!("[EVENT] Internal: Closure failed to be downcasted");
//                         }
//                     }
//                 }
//             } else {
//                 println!("[EVENT EMIT] The event <Event: {:?}> emitting with the data has an invalid data type: {:?}", event, data.clone());
//             }
//         } else {
//             println!("[EVENT EMIT] Event not found: {:?}", event);
//         }
//     }
//
//     // TODO: May want add a tag name for each subscribed observer to easily remove each individual observer later
//
//     // adds a new closure observer that will be executed whenever the relevant event is emitted
//     // subscribes the closure parameter to the event
//     pub fn subscribe<T: FnMut(A), A: 'static>(self: Rc<Self>, event: EventName, closure: T) {
//         if let Some(tid) = self.event_types.get(&event) {
//             if tid == &TypeId::of::<A>() {
//                 // coercing the closure to an Any
//                 let mut func = unsafe {
//                     std::mem::transmute::<Box<dyn FnMut(A) -> ()>, Box<dyn Any>>(Box::new(closure))
//                 };
//
//                 // unsafe {
//                 //     println!("============================== A ==============================");
//                 //     let res = func.downcast_mut::<Box<dyn FnMut(A) -> ()>>();
//                 //     if let None = res {
//                 //         println!("None :/");
//                 //     } else {
//                 //         println!("Value!");
//                 //     }
//                 //     println!("______________________________ B ______________________________");
//                 // }
//
//                 (*self.observers.borrow_mut()).push((event, func));
//             } else {
//                 println!("[EVENT SUBSCRIBE] An observer subscribing to <Event: {:?}> has passed an invalid mutable closure parameter for the respecting event", event);
//             }
//         } else {
//             println!("[EVENT SUBSCRIBE] Event not found: {:?}", event);
//         }
//     }
//
//     // unsubscribes any observers that are subscribed to the selected event to be unsubscribed
//     pub fn unsubscribe(self: Rc<Self>, event: EventName) {
//         (*self.observers.borrow_mut()).retain(|(ename, _)| ename != &event);
//     }
// }
//


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


// pub struct EventDispatcher<'o> {
//     event_types: HashMap<EventName, Vec<TypeId>>,
//     // a list of observers: (The event name it is listening to, the closure it calls when the event is emitted)
//     observers: RefCell<Vec<(EventName, Box<dyn FnMut(EventData) -> () + 'o>)>>,
// }
//
// impl<'o> EventDispatcher<'o> {
//     pub fn new(events: HashMap<EventName, Vec<TypeId>>) -> Rc<Self> {
//         Rc::new(
//             Self {
//                 event_types: events,
//                 observers: RefCell::new(Vec::new()),
//             }
//         )
//     }
//
//     // emits a new events and send it to all of the observers stored in this interchange or connected ones
//     // event is the event name in strings coupled with data that is associated with the event
//     pub fn emit(self: Rc<Self>, event: EventName, mut data: EventData) {
//         // makes sure the event name has been registered within this EventDispatcher
//         if !self.event_types.keys().collect::<Vec<_>>().contains(&&event) {
//             panic!(format!("Event name {:?} is not part of the global struct EventDispatcher", event));
//         }
//         // since the metho `pack()` is not public, we can safely assume that the EventData is not packed
//         // and to further regulate codes from outside
//         data.pack();
//
//         // makes sure the right type has been submitted to `emit()` to be executed on other FnMut()
//         if self.event_types[&event] != data.type_info {
//             panic!(format!("The EventData submitted to `emit()` with the event parameter <{:?}> is invalid on the basis of HashMap<EventName, Vec<TypeId> at instance", event));
//         }
//
//         for (obs_ename, ref mut closure) in &mut *self.observers.borrow_mut() {
//             if &event == obs_ename {
//                 // found observers with its event observing the same with the currently emitting event
//                 // (closure.downcast::<dyn FnMut(T) -> ()>())
//                 (*closure)(data.clone());
//             }
//         }
//     }
//
//     // TODO: May want add a tag name for each subscribed observer to easily remove each individual observer later
//
//     // adds a new closure observer that will be executed whenever the relevant event is emitted
//     // subscribes the closure parameter to the event
//     pub fn subscribe<F: FnMut(EventData) + 'o>(self: Rc<Self>, event: EventName, mut callback: F) {
//         // &mut callback as &mut dyn FnMut(Rc<EventData>)
//         (*self.observers.borrow_mut()).push((event, Box::new(callback)));
//     }
//
//     // unsubscribes any observers that are subscribed to the selected event to be unsubscribed
//     pub fn unsubscribe(self: Rc<Self>, event: EventName) {
//         (*self.observers.borrow_mut()).retain(|(ename, _)| ename != &event);
//     }
// }
//
// fn event_testing() {
//     let event = EventDispatcher::new(types::global_enmtyp());
//     let mut nums = 0;
//
//     let mut ed = EventData::new();
//     ed.push(34u32);
//     event.emit(EventName("SomeEventName"), ed);
//     match event.emitted() {
//         EventName("SomeEventName") => {
//             dt
//         },
//         _ => {},
//     }
//     if Some(dt) = event.emitted(EventName("SomeEventName")) {
//         dt
//     } else if Some(dt) = event.emitted(EventName("SomeEventName")) {
//
//     }
//
// }

//
// struct ObsData {
//     score: u32,
//     stuff: bool,
// }
//
// impl Observer for ObsData {
//     fn event_name(&self) -> EventName {
//         EventName("@TEST")
//     }
//
//     fn receive(&mut self, evd: EventData) -> EventName {
//         unimplemented!()
//     }
//
//     fn retrieve_data(&self) -> EventData {
//         unimplemented!()
//     }
// }
//
// struct World {
//     render: u32,
//     states: bool,
// }
//
// impl World {
//     fn update(&mut self, e: EventDispatcher) {
//         e.notify(EventName("@TEST"), EventData::new())
//     }
// }
//
// impl Subject for World {
//     fn add_observer() {
//         unimplemented!()
//     }
//
//     fn remove_observer() {
//         unimplemented!()
//     }
//
//     fn emit() {
//         unimplemented!()
//     }
// }
//
// pub trait Observer {
//     // the event name you are observing to
//     fn event_name(&self) -> EventName;
//     fn receive(&mut self, evd: EventData) -> EventName;
//     fn retrieve_data(&self) -> EventData;
// }
//
// pub trait Subject {
//     fn add_observer();
//     fn remove_observer();
//     fn emit();
// }
//
// struct EventDispatcher {
//     event_names: HashMap<EventName, Vec<TypeId>>,
//     observer: Vec<Box<dyn Observer>>,
//     subjects: Vec<Box<dyn Subject>>,
// }
//
// impl EventDispatcher {
//     fn new() -> Self {
//         Self {
//             observer: Vec::new(),
//         }
//     }
//
//     fn notify(&self, event: EventName, mut evd: EventData) {
//         ed.pack();
//         let evd = Rc::new(RefCell::new(evd));
//         for o in self.observer {
//             if o.event_name() == event {
//                 o.notify(evd.clone());
//             }
//         }
//     }
//
//     fn subscribe(&mut self, callback: impl Observer) {
//         self.observer.push(Box::new(callback))
//     }
// }
//
// fn etest() {
//     let mut e = EventDispatcher::new();
//     e.subscribe(types::ChunkData);
//     let mut evd = EventData::new();
//     evd.push(false);
//     e.notify(EventName("@TEST/Testing"), g);
// }

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

// fn final_test_event_dispt() {
//     let mut ed = EventDispatcher::new(types::global_enmtyp());
//     ed.clone().emit(EventName("@TEST/Testing"), event_data![32, String::from("Heap-allocated strings"), "&str data"]);
//
//     let dt = vec![String::from("Oh no")];
//
//     let mut var = 45;
//
//     ed.clone().receive(EventName("@TEST/Testing"), |mut evd| {
//         let p_a = evd.pop::<u32>();
//         let p_b = evd.pop::<String>();
//         let p_c = evd.pop::<&str>();
//         println!("P A {:?}", p_a);
//         var += p_a;
//         println!("s1: {:?} and s2: {:?}", p_b, p_c);
//     })
// }

// fn test_for_ed_fourth() {
//     let ed = EventDispatcher::new();
//     ed.emit(EventName, EventData);
//
//     // instead of storing FnMut
//     // the code calls the ed.emission() code on each loop and will be executed
//
//     // to receive and call the closure for all the selected events since last flushed
//     ed.receive(EventName, |e_data| {});
//     // to receive and call the closure once of the next selected event since instance
//     ed.receive_once(EventName, |e_data| {});
//     // to receive and call the closure once of the first selected event since last flushed
//     ed.receive_once_flush(EventName, |e_data| {});
//
//     // **NOTE** To control each receive, you can:
//     ed.receive(EventName, ReceiverName, |e_data| {});
//     ed.disable(ReceiverName);
//     ed.enable(ReceiverName);
//
//     ed.flush();
// }
