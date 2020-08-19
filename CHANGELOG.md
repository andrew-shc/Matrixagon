### Future
* (De)serialization of world
* Entities addition
* Block Entities?
    * Maybe revamp to an ECS system?
* Fully-fledged weather system
* A stack machine to hold the app state (e.g. main menus, options, ...)
* Add update queues to synchronize the game

### Unreleased (Generally ordered from top to bottom)
* [ ] Add a world event system
* [ ] Added block placing and breaking
* [ ] Create an external World Command DSL API interface
* [ ] Create a text file reader to fetch commands into the interface

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
