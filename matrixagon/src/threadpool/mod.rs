
// Design for my custom multithreading

/*

let mut ctp = ChunkThreadpool::new()

ctp is owned by the chunk handler

everything here is under one mesh (param: ctp):
    (world_chunks, chunk id/chunk struct) > ctp.load_newchunk() > (vertices, indices)
    (world_chunks, chunk id/chunk struct) > ctp.load_newchunk() > (vertices, indices)
    (world_chunks, chunk id/chunk struct) > ctp.load_newchunk() > (vertices, indices)
    (world_chunks, chunk id/chunk struct) > ctp.load_newchunk() > (vertices, indices)
    (world_chunks, chunk id/chunk struct) > ctp.load_newchunk() > (vertices, indices)

    let v: Vec<(id, v, i)> = ctp.join()  // generates all the data

    v is a list of chunk data with chunk ID

    then call:

    mesh_0.render(); // and there you go

 */

/*
THreadpool for Rayon

Using threadpool.install to run new threads to pass in the closure/function with required parameters
and then return the vertex and index data.
NOTE: No loops or MPSC or Channels.

 */

use rayon::{ThreadPoolBuilder, ThreadPool};

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::any::Any;


// (Chunk ID, (Vertex Data, Index Data) )
// type ThreadPoolOutput = (ChunkID, (Box<dyn Any>, Box<dyn Any>));
pub type ThreadPoolOutput = Box<dyn Any + Send>;
pub type ThreadPoolInput = Box<dyn FnOnce() -> ThreadPoolOutput + Send>;

// A threadpool using rayon to parallelize the work of chunk mesh generation
pub struct ThreadPoolHandler {
    threadpool: ThreadPool,
    // using String to tag a name for that specific task; a bool to identify the receiver has already received the results back
    channels: Vec<(String, bool, Receiver<ThreadPoolOutput>)>,
}

// TODO: Also handle cancel request for chunkloading
impl ThreadPoolHandler {
    pub fn new(num: usize) -> Self {
        let threadpool = ThreadPoolBuilder::new()
            .num_threads(num)
            // .stack_size(4 * 1024 * 1024)
            .thread_name(|id| format!("Chunk Threadpool: {}", id.to_string()))
            .build().unwrap();

        Self {
            threadpool: threadpool,
            channels: Vec::new(),
        }
    }

    // note: adding work using the same tag but with an FnOnce closure returning different types can cause havoc
    // add your task (FnOnce closure) to the Rayon threadpool
    pub fn add_work<F: FnOnce() -> R + Send + 'static, R: Send + 'static>(&mut self, tag: &'static str, inp: F) {
        // the retrieving data channel
        let (tx, rx) = mpsc::channel();
        self.channels.push((String::from(tag), false, rx));

        self.threadpool.spawn(move || {
            let res = tx.send(Box::new(inp()) as Box<dyn Any + Send>);
            if let Err(e) = res {
                println!("A thread in the chunk mesh generation threadpool has failed to send the data: {}", e);
            }
        });
    }

    // retrieve all the processed result (right now) with the same tag name
    pub fn join_finished<T: 'static>(&mut self, tag: &'static str) -> Vec<T> {
        let mut output_data = Vec::new();

        for (chan_tag, mut recvd,recv) in self.channels.iter_mut() {
            if !recvd && chan_tag == tag {
                if let Ok(dt) = recv.try_recv() {
                    let res = dt.downcast::<T>().expect("Type conversion of function `join_finished()` went wrong");
                    output_data.push(*res);
                    recvd = true;
                }
            }
        }

        // clean-up all received channels
        self.channels.retain(|(_tag,recvd,_recv)| !*recvd);

        output_data
    }

    // retrieve all the processed result regardless whether its finished or not (block until all results come) with the same tag name
    // basically a guarantee that all work added will receive a same lengthened output, unless you used `join_finished()` before this
    pub fn join_block<T: 'static>(&mut self, tag: &'static str) -> Vec<T> {
        let mut output_data = Vec::new();

        for (chan_tag, mut recvd,recv) in self.channels.iter_mut() {
            if !recvd && chan_tag == tag {
                if let Ok(dt) = recv.recv() {
                    let res = dt.downcast::<T>().expect("Type conversion of function `join_finished()` went wrong");
                    output_data.push(*res);
                    recvd = true;
                }
            }
        }

        // clean-up all received channels
        self.channels.retain(|(_tag,recvd,_recv)| !*recvd);

        output_data
    }
}

fn threadpool_testing() {
    let mut thp = ThreadPoolHandler::new(4);
    thp.add_work("Print", move || {
        let mut r = 1u32;
        for i in 0..100 {
            r *= i;
        }
        r
    });
    thp.add_work("Print", move || {
        let mut r = 1;
        for i in 0..100 {
            r *= i;
        }
        r
    });
    let res = thp.join_block::<i32>("Print");

}
