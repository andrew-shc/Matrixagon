# TheEndlessWorld
A game heavily-inspired by the terrain generation of Minecraft and other Minecraft
mods using Rust's Vulkan Wrapper (Vulkano) to construct the game.

![Matrixagon Terrain Gen View](./doc/Matrixagon000_2.png)

### General
This game is a block-based game similiar to Minecraft, but prioritize on block 
simulation but with added minimal gameplay. But when we say minimal gameplay, it 
means there are no crafting, enchanting, and alike. Though there are inventory
systems and player effects.

It specializes in realistic and fantastical terrain generation, weather systems,
machine learning entities, and other components. And hopefully a total modularized
systems with everything customizable from player commands to internal render meshes.

And please do consider this as a game, and not a serious simulation.

### Development Status
This game is still heavily in development. Check [CHANGELOG](CHANGELOG.md) for future plans

Also, please do check this weekly. We'll be updating the game weekly at minimum.

### Installing and Running
This project will not be compiled to binaries as of yet. Though you can compile
the source on your OS to run the program.

NOTE: The following procedure assumes you are on a OS (that also supports Vulkan API).

1. Under the green button of this repo's homepage, click on "Download ZIP"
2. Find the zip file, then extract to your desired location.
    2. Go extract it to "C:/Users/<USERNAME>/Desktop" (on Windows) if you do not know where to extract
3. Open command prompt
4. Type in and enter `cargo init`
5. Type in and enter `cargo run`
    5. This will compile and manage the dependencies all for you
6. A small window should pop-up

### Tutorial / Wiki
#### Keymapping
[W] - Forward  
[A] - Left  
[S] - Back  
[D] - Right  
[LSHIFT] - Downward  
[SPACE] - Upward  

[T] - Escape mouse lock and world.player rotation  
[CTRL] + [W]/[A]/[S]/[D]/[LSHIFT]/[SPACE] - To increase the player movement

*Yet to be implemented*
[L-CLICK] - Break block
[R-CLICK] - Place Block
