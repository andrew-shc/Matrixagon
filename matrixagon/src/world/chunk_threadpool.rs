
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

use crate::world::ChunkID;

use rayon::{ThreadPoolBuilder, ThreadPool};

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::any::Any;


// (Chunk ID, (Vertex Data, Index Data) )
// type ThreadPoolOutput = (ChunkID, (Box<dyn Any>, Box<dyn Any>));
pub type ThreadPoolOutput = (Box<dyn Any + Send>, Box<dyn Any + Send>);
pub type ThreadPoolInput = (ChunkID, Box<dyn FnOnce() -> ThreadPoolOutput + Send>);

// A threadpool using rayon to parallelize the work of chunk mesh generation
pub struct ChunkThreadPool {
    threadpool: ThreadPool,
    channels: Vec<(ChunkID, Receiver<ThreadPoolOutput>)>,
}

// TODO: Also handle cancel request for chunkloading
impl ChunkThreadPool {
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

    // adds a new chunk struct to the thread pool to generate mesh data via closure
    // spawns new threads to the thread pool to generate mesh data via closure
    pub fn add_work(&mut self, inp: ThreadPoolInput) {
        // the retrieving data channel
        let (tx, rx) = mpsc::channel();
        self.channels.push((inp.0, rx));

        self.threadpool.spawn(move || {
            let res = tx.send(inp.1());
            if let Err(e) = res {
                println!("A thread in the chunk mesh generation threadpool has failed to send the data: {}", e);
            }
        });
    }

    // generates chunk data to all the chunks concurrently
    pub fn join(&mut self) -> Vec<(ChunkID, ThreadPoolOutput)> {
        let threads = self.channels.len();
        let mut output_data = Vec::new();

        while threads > output_data.len() {
            // grabs the data for each thread
            for chan in self.channels.iter() {
                match chan.1.try_recv() {
                    Ok(data) => {
                        output_data.push((chan.0, data))
                    },
                    Err(_) => {
                        continue;
                    }
                }
            }
        }

        self.channels.clear();

        output_data
    }
}
