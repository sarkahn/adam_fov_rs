//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//! [![Crates.io](https://img.shields.io/crates/v/adam_fov_rs)](https://crates.io/crates/adam_fov_rs)
//! [![docs](https://docs.rs/adam_fov_rs/badge.svg)](https://docs.rs/adam_fov_rs/)
//!
//! An implementation of [Adam Millazo's FOV algorithm](http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine)
//!
//! This crate provides a single function, `compute_fov`, which computes a field
//! of view into a 2d grid from existing map data.
//!
//! # Example
//!
//! ```rust
//! use adam_fov_rs::*;
//!
//! #[derive(Clone)]
//! enum Tile {
//!     Floor,
//!     Wall,
//! }
//! let width = 50;
//! let height = 50;
//! let index = |p: IVec2| p.y as usize * width + p.x as usize;
//!
//! let mut game_map = vec![Tile::Floor; width * height];
//! // Add a vision blocker
//! game_map[index(IVec2::new(15, 16))] = Tile::Wall;
//!
//! let mut vision = vec![false; width * height];
//!
//! let is_opaque = |p| matches!(game_map[index(p)], Tile::Wall);
//! let mark_visible = |p| {
//!     let i = index(p);
//!     vision[i] = true;
//! };
//!
//! compute_fov([15, 15], 5, [width, height], is_opaque, mark_visible);
//!
//! let is_visible = |p| vision[index(p)];
//! assert!(!is_visible(IVec2::new(15, 17)));
//! assert!(is_visible(IVec2::new(17, 15)));
//! ```
//!
//! *Taken from the terminal example*
//! ![](images/fov.gif)

pub use glam::IVec2;
pub use sark_grids::{GridPoint, GridSize};

/// Compute a field of view into a 2d grid from existing map data.
///
/// This algorithm assumes your map is a a 2d grid of tiles where each tile can
/// be either transparent or opaque.
///
/// Note that the map data that defines which tiles are opaque is assumed to be
/// seperate from the visibility data which describes a view into that map.
///
/// # Arguments
///
/// * `origin` and `range` - Define the circular area of the grid that will be used
///   to calculate the field of view.
///
/// * `grid_size` - Used for bounds checking during fov calculation.
///
/// * `tile_blocks_vision` - A callback which should return true if the tile at a
///   given map position is opaque, meaning it blocks vision.
///
/// * `mark_tile_visible` - A callback that will be called during fov calculation
///   to notify the caller that a map tile is visible. This might be called
///   multiple times for the same tile.
///
/// # Example
///
/// ```rust
/// use adam_fov_rs::*;
///
/// #[derive(Clone)]
/// enum Tile {
///     Floor,
///     Wall,
/// }
/// let width = 50;
/// let height = 50;
/// let index = |p: IVec2| p.y as usize * width + p.x as usize;
///
/// let mut game_map = vec![Tile::Floor; width * height];
/// // Add a vision blocker
/// game_map[index(IVec2::new(15, 16))] = Tile::Wall;
///
/// // Describes which map tiles are visible or not
/// let mut vision = vec![false; width * height];
///
/// let is_opaque = |p| matches!(game_map[index(p)], Tile::Wall);
/// let mark_as_visible = |p| {
///     let i = index(p);
///     vision[i] = true;
/// };
///
/// compute_fov([15, 15], 5, [width, height], is_opaque, mark_as_visible);
///
/// let is_visible = |p| vision[index(p)];
/// assert!(!is_visible(IVec2::new(15, 17)));
/// assert!(is_visible(IVec2::new(17, 15)));
/// ```
pub fn compute_fov(
    origin: impl GridPoint,
    range: usize,
    max_bounds: impl GridSize + Copy, // TODO: Gridsize should implement Copy
    tile_blocks_vision: impl Fn(IVec2) -> bool,
    mut mark_tile_visible: impl FnMut(IVec2),
) {
    let origin = origin.to_ivec2();
    mark_tile_visible(origin);

    for octant in 0..8 {
        compute_octant(
            octant,
            origin,
            range as i32,
            1,
            Slope { x: 1, y: 1 },
            Slope { x: 1, y: 0 },
            max_bounds,
            &tile_blocks_vision,
            &mut mark_tile_visible,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_octant(
    octant: i32,
    origin: IVec2,
    range: i32,
    x: i32,
    mut top: Slope,
    mut bottom: Slope,
    grid_size: impl GridSize + Copy,
    is_opaque: &impl Fn(IVec2) -> bool,
    mark_tile_visible: &mut impl FnMut(IVec2),
) {
    for x in x..=range {
        let y_coords = compute_y_coordinate(
            octant,
            origin,
            x,
            &mut top,
            &mut bottom,
            grid_size,
            is_opaque,
        );

        let top_y = y_coords.x;
        let bottom_y = y_coords.y;

        if !compute_visiblity(
            top_y,
            bottom_y,
            range,
            octant,
            origin,
            x,
            &mut top,
            &mut bottom,
            grid_size,
            is_opaque,
            mark_tile_visible,
        ) {
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_y_coordinate(
    octant: i32,
    origin: IVec2,
    x: i32,
    top: &mut Slope,
    bottom: &mut Slope,
    grid_size: impl GridSize + Copy,
    is_opaque: &impl Fn(IVec2) -> bool,
) -> IVec2 {
    let mut top_y;
    if top.x == 1 {
        top_y = x;
    } else {
        top_y = ((x * 2 - 1) * top.y + top.x) / (top.x * 2);

        if blocks_light(x, top_y, octant, origin, grid_size, is_opaque) {
            if top.greater_or_equal(top_y * 2 + 1, x * 2)
                && !blocks_light(x, top_y + 1, octant, origin, grid_size, is_opaque)
            {
                top_y += 1;
            }
        } else {
            let mut ax = x * 2;
            if blocks_light(x + 1, top_y + 1, octant, origin, grid_size, is_opaque) {
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
            && blocks_light(x, bottom_y, octant, origin, grid_size, is_opaque)
            && !blocks_light(x, bottom_y + 1, octant, origin, grid_size, is_opaque)
        {
            bottom_y += 1;
        }
    }
    IVec2::new(top_y, bottom_y)
}

fn blocks_light(
    x: i32,
    y: i32,
    octant: i32,
    origin: IVec2,
    grid_size: impl GridSize,
    is_opaque: &impl Fn(IVec2) -> bool,
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
    if !grid_size.contains_point(p) {
        return true;
    }
    is_opaque(IVec2::new(nx, ny))
}

#[allow(clippy::too_many_arguments)]
fn compute_visiblity(
    top_y: i32,
    bottom_y: i32,
    range: i32,
    octant: i32,
    origin: IVec2,
    x: i32,
    top: &mut Slope,
    bottom: &mut Slope,
    grid_size: impl GridSize + Copy,
    is_tile_opaque: &impl Fn(IVec2) -> bool,
    mark_tile_visible: &mut impl FnMut(IVec2),
) -> bool {
    let mut was_opaque = -1;

    for y in (bottom_y..=top_y).rev() {
        if range < 0 || glam::Vec2::ZERO.distance(IVec2::new(x, y).as_vec2()) <= range as f32 {
            let is_opaque = blocks_light(x, y, octant, origin, grid_size, is_tile_opaque);

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
                set_visible(x, y, octant, origin, grid_size, mark_tile_visible);
            }

            if x != range {
                if is_opaque {
                    if was_opaque == 0 {
                        let mut nx = x * 2;
                        let ny = y * 2 + 1;
                        if blocks_light(x, y + 1, octant, origin, grid_size, is_tile_opaque) {
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
                                    grid_size,
                                    is_tile_opaque,
                                    mark_tile_visible,
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
                        if blocks_light(x + 1, y + 1, octant, origin, grid_size, is_tile_opaque) {
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

fn set_visible(
    x: i32,
    y: i32,
    octant: i32,
    origin: IVec2,
    grid_size: impl GridSize + Copy,
    mark_tile_visible: &mut impl FnMut(IVec2),
) {
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
    if grid_size.contains_point(p) {
        mark_tile_visible(p);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn testfov() {
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
    }
}
