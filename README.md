[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/adam_fov_rs)](https://crates.io/crates/adam_fov_rs)
[![docs](https://docs.rs/adam_fov_rs/badge.svg)](https://docs.rs/adam_fov_rs/)

An implementation of [Adam Millazo's FOV algorithm](http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine)

# Example

```rust
use adam_fov_rs::*;

// Create a 50x50 visibility map
let mut map = VisibilityMap::new([50,50]);

// Add a vision blocking tile
map.add_blocker([15,15]);
 
// Compute a field of view from a position
map.compute([15,14], 5);

// The tile above our blocker is not visible
assert!( !map.is_visible([15,16]) );
```

![](images/fov.gif)
*Taken from the "terminal" example*