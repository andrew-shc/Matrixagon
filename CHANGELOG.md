### Future
* (De)serialization of world
* Entities addition
* Block Entities?
    * Maybe revamp to an ECS system?
* Fully-fledged weather system
* A stack machine to hold the app state (e.g. main menus, options, ...)
* Add update queues to synchronize the game
* Three major backend components:
    * Event system
    * Global threadpool
    * Frontend component states (struct)

### Unreleased (Generally ordered from top to bottom)
- Added a world event system
- Added block placing and breaking
- Render a chunk first to start the render data than render the rest of it?

* [ ] Create an external World Command DSL API interface
* [ ] Create a text file reader to fetch commands into the interface

* Add structures
* Block updates
* Stop chunk generation if not needed while it is generating
* Fix the negative perline noise issue
* Smarter chunk generation: generate first where the player is standing the closest
* Options for faster chunk generations:
    * Save loaded chunk datas to disk instead of memory
    * Using GPU to generate chunk datas
    * Thread #'s
* Generate chunks in groups

* Multithreading: Use minimal sub-system threads, and use worker threadpools

* Faster chunk loading on the chunk thread
* Render Optimization:
    * Texture mipmapping (claims it will be supported soon for `from_iter` method)
    * Frustum Culling
    * Occlusion Culling
        * Remove all the world.block sides player won't see from the player's position
    * Improve Block Handling
* Wireframe rendering debugging option
* Move the rendering of the world always to the origin of the render space
 to improve floating calculations
* Add a proper event system
* Add basic UI
* Add a terrain generation
* Internal:
    * Add a global shared reference on block registry (Arc<T>)

### Future v0.1.6
* A new separate interface for using commands
    * An interface to modify world data, entities, and the game itself
    * Uses the event systems
* Using threadpools (for scalability performance) comapred to the original sub-system threading
    - A global threadpool instantiated in the beginning
    - Push a new worker task using fn/FnOnce closures to execute and complete the task
    - Then either a choice after submission: 
            - Awaiting all tasks to be completed; blocking main thread
            - Await all tasks to be completed at a later time; postpone blocking main thread
            - Notify/Check once all tasks are completed
            - Cancel tasks
            - Panic when a task has panicked
     - Each task has a status of: Processing, Postponed, Idle
     - And a action of: Submit (create a new task), Postpone, Cancel
* The renderbuffer will be updated for each chunk rendered
    * Unlike the previous version where we wait for a whole section to finish loading
    * Gives a more performant point of view

### v0.1.5 [Oct 4, 2020]
* Added world event to organize code and easily manage events
    * Event System:
        - A globalized event systems with a queue of classified events
        - Creates a new event bus
        - Then have some way of emitting events to the bus to all the receivers
        - Then the bus on the other subscribed to a (multiple) specific events gets executed
* Command bytecodes are compiling properly
* Removed sub-system threading yielding a slower performance temporarily for now
    * Will be using threadpool in next version

### v0.1.4 [Sept 3, 2020]
* Chunk mesh generation improvement
    * Using layers to lookup for any transparent blocks inside the chunk
    * Only regenerate chunks that are next to newly generated chunks <-- me totally stupid
        * Really did not improved chunk mesh speed
* Added a (unusable) World Command Bytecode Language
    * planned to be executed and change the world
* Re-organized the project repo for an upcoming separate crate
    * This crate is the World Command Scripting Language
    * Its where this language compiles down to this bytecode
* Specification for the command bytecode will be released later this year
* NOTE: This is taking longer than expected
* LOG: Project file and resource/doc file corrupted
    * Had to download the files from my repo on github
    * Somehow my air.png file was all ok
    
### v0.1.3 [Aug 23, 2020] (The most fruitful update so far :D)
* __Fixed__: Minimizing window crashes the app because of Dimension {0,0}
    * currently halts the app when the window is minimized
* Added basic block lighting system
* Added new mesh: Flora; to add some grass and flowers into the terrain
* Added sand blocks to the terrain generation
* Added Perlin noise for heightmap generation
* Prevent chunk loading below y-level 0
* Internal:
    * Changed the mesh of the Air block from Cube -> Air mesh; a noticeable
     improvement on the performance of chunk loading
    * Refactored the texture manager to hold `sampler2DArray` instead of `sampler2D[]`.
    This should allow adding new texture much more dynamic and easier (with still almost same performance)
    * Codebase deep clean
    
### v0.1.2 [Aug 19, 2020]
* [x] __Fixed__: Graphic pipeline does not update to a new size of the window in a reasonable amount of time; and now will be immediately update
* [X] Significantly improved chunk loading/generation
* [X] Added basic noise terrain height map
* [x] Re-tuned the player movement speed
    * [x] Added new [CTRL] key-binding to increase player movement
    * NOTE: The movement increase will be significantly faster once the chunk loading is optimized and refined more
* Internal:
    * Codebase cleanup and reorganization
    * Minimized warnings
    * Changed struct/method types
    * Improved buffer data spamming handling over multithreading MPSC channels
    * [X] Added block registry for more globalize/easier way to obtain block data

### v0.1.1 [Aug 8, 2020]
* Render Optimization:
    * Chunk Border Culling
    * Placed synchronized chunk loading into a separate thread
* (Internal) Added Block and Chunk position units
    * 1 Block [1bc] (Basic Unit)
    * 1 Chunk [1ch] = 32 Block
    * 1 Sector [1sc] = 16 Chunks

### v0.1.0 [June 26, 2020]
* Finished refactoring, similar to the previous repository: Mineblock
* Significant code cleanup and re-organization
* Using `nalgebra` crate instead of `cgmath` crate
* From previous updates:
    * Player movement is fixed
    * Texture selection should be deterministic
    * Oriented the texture properly
    * Automatic chunk loading
* Updated `vulkano 0.18.0` to `vulkano 0.19.0`
* Updated `vulkano-win 0.18.0` to `vulkano-win 0.19.0`
* Updated `vulkano-shaders 0.13.0` to `vulkano-shaders 0.19.0`

### v0.0.2 [June 20, 2020]
* Partially finished refactoring
* WARNING: weird player translation
* NOTE: just want to pushed this out b/c its just taking longer than expected

### v0.0.1 [June 4, 2020]
* This will be the heavily-refactored version from the previous repository, Mineblock
* There WILL be a lot of internal code changes and type changes
* To easily support multi-threading
