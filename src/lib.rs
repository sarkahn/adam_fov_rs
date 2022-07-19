//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//! [![Crates.io](https://img.shields.io/crates/v/adam_fov_rs)](https://crates.io/crates/adam_fov_rs)
//! [![docs](https://docs.rs/adam_fov_rs/badge.svg)](https://docs.rs/adam_fov_rs/)
//!
//! An implementation of [Adam Millazo's FOV algorithm](http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine)
//!
//! To use it you must implement the [VisibilityMap] trait on your map type, or use
//! the build in [VisibilityMap2d]. Then you can call [fov::compute] with your map
//! which will populate visible tiles based on the map's opaque tiles.
//!
//! # Example
//! ```rust
//! use adam_fov_rs::*;
//!
//! // Create a 50x50 visibility map
//! let mut map = VisibilityMap2d::default([50,50]);
//!
//! // Set the tile at (15,15) to opaque
//! map[[15,15]].opaque = true;
//!
//! // Compute our visible tiles and add them to the map
//! fov::compute([15,14], 5, &mut map);
//!
//! // The space directly above our opaque tile is not visible
//! assert!(map[[15,16]].visible == false);
//! ```
//!
//! *Taken from the terminal example*
//! ![](images/fov.gif)

pub use glam::IVec2;
use glam::Vec2;
use sark_grids::Grid;
pub use sark_grids::GridPoint;

pub type VisibilityMap2d = Grid<VisibilityPoint>;

/// A trait used by the fov algorithm to calculate the resulting fov.
pub trait VisibilityMap {
    fn is_opaque(&self, p: impl GridPoint) -> bool;
    fn is_in_bounds(&self, p: impl GridPoint) -> bool;
    fn set_visible(&mut self, p: impl GridPoint);
    fn dist(&self, a: impl GridPoint, b: impl GridPoint) -> f32;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VisibilityPoint {
    pub visible: bool,
    pub opaque: bool,
}

impl VisibilityMap for VisibilityMap2d {
    fn is_opaque(&self, p: impl GridPoint) -> bool {
        if self.in_bounds(p) {
            self[p].opaque
        } else {
            true
        }
    }

    fn is_in_bounds(&self, p: impl GridPoint) -> bool {
        self.in_bounds(p)
    }

    fn set_visible(&mut self, p: impl GridPoint) {
        if self.in_bounds(p) {
            self[p].visible = true;
        }
    }

    fn dist(&self, a: impl GridPoint, b: impl GridPoint) -> f32 {
        Vec2::distance(a.as_vec2(), b.as_vec2())
    }
}

pub trait VisibilityMapUtility {
    fn toggle_opaque(&mut self, p: impl GridPoint);
    fn toggle_visible(&mut self, p: impl GridPoint);
    fn clear_opaque(&mut self);
    fn clear_visible(&mut self);
}

impl VisibilityMapUtility for VisibilityMap2d {
    fn toggle_opaque(&mut self, p: impl GridPoint) {
        let i = self.pos_to_index(p);
        self[i].opaque = !self[i].opaque;
    }

    fn toggle_visible(&mut self, p: impl GridPoint) {
        let i = self.pos_to_index(p);
        self[i].visible = !self[i].visible;
    }

    /// Clear all opaque tiles from the map
    fn clear_opaque(&mut self) {
        self.iter_mut().for_each(|p| p.opaque = false);
    }

    /// Clear all visible tiles from the map
    fn clear_visible(&mut self) {
        self.iter_mut().for_each(|p| p.visible = false);
    }
}

/// Module containing the compute function.
pub mod fov {
    use glam::IVec2;

    use crate::{GridPoint, VisibilityMap};

    /// Compute the fov in a map from the given position.
    pub fn compute<T: VisibilityMap>(origin: impl GridPoint, range: i32, map: &mut T) {
        let origin = origin.as_ivec2();
        map.set_visible(origin);

        for octant in 0..8 {
            compute_octant(
                octant,
                origin,
                range,
                1,
                Slope { x: 1, y: 1 },
                Slope { x: 1, y: 0 },
                map,
            )
        }
    }

    fn compute_octant<T: VisibilityMap>(
        octant: i32,
        origin: IVec2,
        range: i32,
        x: i32,
        mut top: Slope,
        mut bottom: Slope,
        map: &mut T,
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

    fn compute_y_coordinate<T: VisibilityMap>(
        octant: i32,
        origin: IVec2,
        x: i32,
        map: &mut T,
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

    #[allow(clippy::too_many_arguments)]
    fn compute_visiblity<T: VisibilityMap>(
        top_y: i32,
        bottom_y: i32,
        range: i32,
        octant: i32,
        origin: IVec2,
        x: i32,
        map: &mut T,
        top: &mut Slope,
        bottom: &mut Slope,
    ) -> bool {
        let mut was_opaque = -1;

        for y in (bottom_y..=top_y).rev() {
            if range < 0 || map.dist(IVec2::ZERO, IVec2::new(x, y)) <= range as f32 {
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

    fn blocks_light<T: VisibilityMap>(
        x: i32,
        y: i32,
        octant: i32,
        origin: IVec2,
        map: &mut T,
    ) -> bool {
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
        if !map.is_in_bounds(p) {
            return true;
        }
        map.is_opaque(IVec2::new(nx, ny))
    }

    fn set_visible<T: VisibilityMap>(x: i32, y: i32, octant: i32, origin: IVec2, map: &mut T) {
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
        if map.is_in_bounds(p) {
            map.set_visible(p);
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
}

#[cfg(test)]
mod test {

    use crate::*;

    #[test]
    fn test_fov() {
        let mut map = VisibilityMap2d::default([30, 30]);
        map[[0, 1]].opaque = true;
        map[[1, 0]].opaque = true;
        fov::compute([0, 0], 5, &mut map);

        assert!(map[[0, 0]].visible);

        assert!(map[[0, 1]].visible);
        assert!(!map[[0, 2]].visible);

        assert!(map[[1, 0]].visible);
        assert!(!map[[2, 0]].visible);
    }
}
