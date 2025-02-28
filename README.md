[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/adam_fov_rs)](https://crates.io/crates/adam_fov_rs)
[![docs](https://docs.rs/adam_fov_rs/badge.svg)](https://docs.rs/adam_fov_rs/)

An implementation of [Adam Millazo's FOV algorithm](http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine)

This crate provides a single function, `compute_fov` which can be used to compute a field of view into a 2d grid from existing map data. `compute_fov` uses function callbacks to read map data and define visible tiles for the caller. 

# Example

```rust
use adam_fov_rs::*;

#[derive(Clone)]
enum Tile {
    Floor,
    Wall,
}

let width = 50;
let height = 50;
let index = |p: IVec2| p.y as usize * width + p.x as usize;

let mut game_map = vec![Tile::Floor; width * height];
// Add a vision blocker
game_map[index(IVec2::new(15, 16))] = Tile::Wall;

let mut vision = vec![false; width * height];

let is_opaque = |p| matches!(game_map[index(p)], Tile::Wall);
let mark_visible = |p| {
    let i = index(p);
    vision[i] = true;
};

compute_fov([15, 15], 5, [width, height], is_opaque, mark_visible);

let is_visible = |p| vision[index(p)];
assert!(!is_visible(IVec2::new(15, 17)));
assert!(is_visible(IVec2::new(17, 15)));
```

---

![](images/fov.gif)

*Taken from the "terminal" example*