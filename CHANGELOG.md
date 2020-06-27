### Unreleased
* Using multi-threading to push the chunk loading off the main/rendering thread

* Render Optimization:
    * Texture mipmapping (claims it will be supported soon for `from_iter` method)
    * Frustum Culling
    * Occlusion Culling
    * Improve Block Handling

### v0.1.0 [June 24, 2020]
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
