//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//! [![Crates.io](https://img.shields.io/crates/v/adam_fov_rs)](https://crates.io/crates/adam_fov_rs)
//! [![docs](https://docs.rs/adam_fov_rs/badge.svg)](https://docs.rs/adam_fov_rs/)
//!
//! An implementation of [Adam Millazo's FOV algorithm](http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine)
//!
//! # Example
//! ```rust
//! use adam_fov_rs::*;
//!
//! let mut map = VisibilityMap::new([50,50]);
//!
//! map.add_blocker([15,15]);
//!
//! // Compute the field of view
//! map.compute([15,14], 5);
//!
//! // The space directly above our opaque tile is not visible
//! assert!(map.is_visible([15,16]) == false);
//! ```
//!
//! *Taken from the terminal example*
//! ![](images/fov.gif)

use glam::{IVec2, Vec2};
use sark_grids::{BitGrid, GridPoint, GridSize, SizedGrid};

/// A visibility map that can be used to compute a field of view on a 2d grid.
///
/// # Example
/// ```rust
/// use adam_fov_rs::*;
///
/// let mut map = VisibilityMap::new([50,50]);
///
/// map.add_blocker([15,15]);
/// map.compute([15,14], 5);
///
/// // The space behind the vision blocker is not visible
/// assert!(map.is_visible([15,16]) == false);
/// ```
pub struct VisibilityMap {
    visible: BitGrid,
    vision_blocker: BitGrid,
}

impl SizedGrid for VisibilityMap {
    fn size(&self) -> glam::UVec2 {
        self.visible.size()
    }
}

impl VisibilityMap {
    pub fn new(size: impl GridSize) -> Self {
        Self {
            visible: BitGrid::new(size.to_uvec2()),
            vision_blocker: BitGrid::new(size),
        }
    }

    pub fn is_blocker(&self, p: impl GridPoint) -> bool {
        self.vision_blocker.get(p)
    }

    /// Check if a tile is visible within this FOV. This is only valid after
    /// calling `compute`
    pub fn is_visible(&self, p: impl GridPoint) -> bool {
        self.visible.get(p)
    }

    pub fn add_blocker(&mut self, p: impl GridPoint) {
        self.vision_blocker.set(p, true);
    }

    pub fn remove_blocker(&mut self, p: impl GridPoint) {
        self.vision_blocker.set(p, false);
    }

    pub fn toggle_blocker(&mut self, p: impl GridPoint) {
        self.vision_blocker.toggle(p);
    }

    pub fn set_blocker(&mut self, p: impl GridPoint, blocks_vision: bool) {
        self.vision_blocker.set(p, blocks_vision);
    }

    /// Reset previously computed visibility
    pub fn clear_visibility(&mut self) {
        self.visible.clear();
    }

    pub fn clear_blockers(&mut self) {
        self.vision_blocker.clear();
    }

    /// Compute the FOV
    pub fn compute(&mut self, p: impl GridPoint, range: i32) {
        let origin = p.to_ivec2();
        self.visible.set(origin, true);

        for octant in 0..8 {
            compute_octant(
                octant,
                origin,
                range,
                1,
                Slope { x: 1, y: 1 },
                Slope { x: 1, y: 0 },
                self,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_visiblity(
    top_y: i32,
    bottom_y: i32,
    range: i32,
    octant: i32,
    origin: IVec2,
    x: i32,
    map: &mut VisibilityMap,
    top: &mut Slope,
    bottom: &mut Slope,
) -> bool {
    let mut was_opaque = -1;

    for y in (bottom_y..=top_y).rev() {
        if range < 0 || Vec2::ZERO.distance(IVec2::new(x, y).as_vec2()) <= range as f32 {
            let is_opaque = blocks_light(x, y, octant, origin, map);

            // Less symmetrical
            // let is_visible = is_opaque ||
            // (
            //     (y != top_y || top.greater(y * 4 - 1, x * 4 + 1)) &&
            //     (y != bottom_y || bottom.less(y * 4 + 1, x * 4 - 1))
            // );

            // Better symmetry
            let is_visible = is_opaque || // Remove is_opaque check for full symmetry but more artifacts in hallways
                (
                    (y != top_y || top.greater_or_equal(y, x)) &&
                    (y != bottom_y || bottom.less_or_equal(y, x))
                );

            if is_visible {
                set_visible(x, y, octant, origin, map);
            }

            if x != range {
                if is_opaque {
                    if was_opaque == 0 {
                        let mut nx = x * 2;
                        let ny = y * 2 + 1;
                        if blocks_light(x, y + 1, octant, origin, map) {
                            nx -= 1;
                        }
                        if top.greater(ny, nx) {
                            if y == bottom_y {
                                *bottom = Slope { y: ny, x: nx };
                                break;
                            } else {
                                compute_octant(
                                    octant,
                                    origin,
                                    range,
                                    x + 1,
                                    top.clone(),
                                    Slope { y: ny, x: nx },
                                    map,
                                );
                            }
                        } else if y == bottom_y {
                            return false;
                        }
                    }
                    was_opaque = 1;
                } else {
                    if was_opaque > 0 {
                        let mut nx = x * 2;
                        let ny = y * 2 + 1;
                        if blocks_light(x + 1, y + 1, octant, origin, map) {
                            nx += 1;
                        }
                        if bottom.greater_or_equal(ny, nx) {
                            return false;
                        }
                        *top = Slope { y: ny, x: nx };
                    }
                    was_opaque = 0;
                }
            }
        }
    }

    was_opaque == 0
}

fn compute_octant(
    octant: i32,
    origin: IVec2,
    range: i32,
    x: i32,
    mut top: Slope,
    mut bottom: Slope,
    map: &mut VisibilityMap,
) {
    for x in x..=range {
        let y_coords = compute_y_coordinate(octant, origin, x, map, &mut top, &mut bottom);

        let top_y = y_coords.x;
        let bottom_y = y_coords.y;

        if !compute_visiblity(
            top_y,
            bottom_y,
            range,
            octant,
            origin,
            x,
            map,
            &mut top,
            &mut bottom,
        ) {
            break;
        }
    }
}

fn compute_y_coordinate(
    octant: i32,
    origin: IVec2,
    x: i32,
    map: &mut VisibilityMap,
    top: &mut Slope,
    bottom: &mut Slope,
) -> IVec2 {
    let mut top_y;
    if top.x == 1 {
        top_y = x;
    } else {
        top_y = ((x * 2 - 1) * top.y + top.x) / (top.x * 2);

        if blocks_light(x, top_y, octant, origin, map) {
            if top.greater_or_equal(top_y * 2 + 1, x * 2)
                && !blocks_light(x, top_y + 1, octant, origin, map)
            {
                top_y += 1;
            }
        } else {
            let mut ax = x * 2;
            if blocks_light(x + 1, top_y + 1, octant, origin, map) {
                ax += 1;
            }
            if top.greater(top_y * 2 + 1, ax) {
                top_y += 1;
            }
        }
    }

    let mut bottom_y;
    if bottom.y == 0 {
        bottom_y = 0;
    } else {
        bottom_y = ((x * 2 - 1) * bottom.y + bottom.x) / (bottom.x * 2);

        if bottom.greater_or_equal(bottom_y * 2 + 1, x * 2)
            && blocks_light(x, bottom_y, octant, origin, map)
            && !blocks_light(x, bottom_y + 1, octant, origin, map)
        {
            bottom_y += 1;
        }
    }
    IVec2::new(top_y, bottom_y)
}

fn blocks_light(x: i32, y: i32, octant: i32, origin: IVec2, map: &mut VisibilityMap) -> bool {
    let (mut nx, mut ny) = origin.into();
    match octant {
        0 => {
            nx += x;
            ny -= y;
        }
        1 => {
            nx += y;
            ny -= x;
        }
        2 => {
            nx -= y;
            ny -= x;
        }
        3 => {
            nx -= x;
            ny -= y;
        }
        4 => {
            nx -= x;
            ny += y;
        }
        5 => {
            nx -= y;
            ny += x;
        }
        6 => {
            nx += y;
            ny += x;
        }
        7 => {
            nx += x;
            ny += y;
        }
        _ => {}
    }
    let p = IVec2::new(nx, ny);
    if !map.in_bounds(p) {
        return true;
    }
    map.is_blocker(IVec2::new(nx, ny))
}

fn set_visible(x: i32, y: i32, octant: i32, origin: IVec2, map: &mut VisibilityMap) {
    let (mut nx, mut ny) = origin.into();
    match octant {
        0 => {
            nx += x;
            ny -= y;
        }
        1 => {
            nx += y;
            ny -= x;
        }
        2 => {
            nx -= y;
            ny -= x;
        }
        3 => {
            nx -= x;
            ny -= y;
        }
        4 => {
            nx -= x;
            ny += y;
        }
        5 => {
            nx -= y;
            ny += x;
        }
        6 => {
            nx += y;
            ny += x;
        }
        7 => {
            nx += x;
            ny += y;
        }
        _ => {}
    }
    let p = IVec2::new(nx, ny);
    if map.in_bounds(p) {
        map.visible.set(p, true);
    }
}

#[derive(Clone)]
struct Slope {
    x: i32,
    y: i32,
} // represents the slope Y/X as a rational number

impl Slope {
    // this > y/x
    pub fn greater(&self, y: i32, x: i32) -> bool {
        self.y * x > self.x * y
    }

    // s >= y/x
    pub fn greater_or_equal(&self, y: i32, x: i32) -> bool {
        self.y * x >= self.x * y
    }

    // s < y/x
    //pub fn less(&self, y: i32, x: i32) -> bool {
    //    self.y * x < self.x * y
    //}

    pub fn less_or_equal(&self, y: i32, x: i32) -> bool {
        self.y * x <= self.x * y
    } // this <= y/x
}
