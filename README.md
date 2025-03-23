# Trowback

A simple 2D game built with Rust and the Bevy game engine.

## Description

Trowback is a minimalist 2D game featuring a player-controlled white circle that can be moved around the screen.

## Project Structure

```
trowback/
├── Cargo.toml        # Project configuration and dependencies
├── Cargo.lock        # Dependency lockfile (managed by Cargo)
├── README.md         # This file
└── src/
    └── main.rs       # Game source code
```

## Getting Started

### Prerequisites

- Rust and Cargo installed ([Install Rust](https://www.rust-lang.org/tools/install))

### Running the Game

```bash
# Clone the repository
git clone https://github.com/bybunni/trowback.git
cd trowback

# Run the game
cargo run
```

## Controls

- **W**: Move up
- **A**: Move left
- **S**: Move down
- **D**: Move right

## Development

The game uses Bevy's ECS (Entity Component System) architecture:
- The main player entity is a white circle with the `Player` component
- Movement logic is handled in the `move_player` system

### Build Optimization

The project is configured with development optimizations:
- `opt-level = 1` for local code to maintain fast compilation times
- `opt-level = 3` for dependencies to improve runtime performance

For release builds, remove the `dynamic_linking` feature and use:
```bash
cargo run --release
```
