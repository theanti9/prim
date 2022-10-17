Prim
=====

Prim is a game engine designed to make it easy to build games with just basic shapes.

**Note: Prim is a hobby project, built mostly for learning, and is not even a little production ready.

Features
---------
Here is a non-comprehensive list of what currently exists and what may exist in the future

- [ ] Rendering
    - [x] 2D shape rendering
    - [x] Text rendering
    - [ ] Lights
- [x] ECS (Thanks [Bevy](https://bevyengine.org/)!)
- [ ] Game Systems
    - [x] Particle Systems
    - [x] Collision System
    - [ ] Animation
- [x] Input
    - [x] Keyboard Input
    - [x] Mouse Input
- [ ] Audio
- [ ] Loading
    - [ ] Shapes files

Examples
--------
Examples may be run via cargo with `cargo run --example <feature name>`.

The following examples are currently usable:

### Space Invaders (`space_invaders`)

```
cargo run --example space_invaders
```

This is a small, not entirely complete clone of space invaders to demonstrate and test most of the practical bits of making a small game in the engine. 

![Space Invaders](/screenshots/space_invaders_example.png?raw=true)

### Particle System (`particle_system`)

```
cargo run --example particle_system
```

This is an example showing off the particle systems and their performance. It can handle 10's of thousands of particles and high FPS.

![Particle System](/screenshots/particle_system_example.png?raw=true)

### Input (`input`)
```
cargo run --example input
```

This is an example that displays input codes as a way of displaying how to read keyboard and mouse input as well as display text to the screen.

### Running

```
cargo run
```

Running the main binary itself is currently just a render test, as shown below.

![Render Test](/screenshots/run.png?raw=true)