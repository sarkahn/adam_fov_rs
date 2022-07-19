[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/adam_fov_rs)](https://crates.io/crates/adam_fov_rs)
[![docs](https://docs.rs/adam_fov_rs/badge.svg)](https://docs.rs/adam_fov_rs/)

An implementation of [Adam Millazo's FOV algorithm](http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine)

To use it you must implement the `VisibilityMap` trait on your map type, or use the built in `VisibilityMap2d`. Then you can call `fov::compute` with your map which will populate visible tiles based on the map's opaque tiles.

# Example

```rust
use adam_fov_rs::*;

// Create a 50x50 visibility map
let mut map = VisibilityMap2d::default([50,50]);

// Set the tile at (15,15) to opaque
map[[15,15]].opaque = true;
 
// Compute our visible tiles and add them to the map
fov::compute([15,14], 5, &mut map);

// The space directly above our opaque tile is not visible
assert!(map[[15,16]].visible == false);
```

![](images/fov.gif)

*Taken from the "terminal" example*