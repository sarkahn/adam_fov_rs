//! 
//! 
//! An implementation of Adam Millazo's FOV algorithm from http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html#mine
use glam::IVec2;

pub trait VisiblityMap
{
    fn is_opaque(&self, p: IVec2) -> bool;
    fn is_in_bounds(&self, p: IVec2) -> bool;
    fn set_visible(&mut self, p: IVec2);
    fn dist(&self, a: IVec2, b: IVec2) -> f32;
}

pub mod fov {
    use glam::IVec2;

    use crate::VisiblityMap;

    pub fn compute<T: VisiblityMap>(origin: IVec2, range: i32, map: &mut T) {
        map.set_visible(origin);

        for octant in 0..8 {
            compute_octant(
                octant, origin, range, 1, 
                Slope { x: 1, y: 1 },
                Slope { x: 1, y: 0 },
                map 
            )
        }
    }

    fn compute_octant<T: VisiblityMap>(
        octant: i32, 
        origin: IVec2, 
        range: i32, 
        mut x: i32, 
        mut top: Slope, 
        mut bottom: Slope, 
        map: &mut T
    ) {
        for _ in x..= range {
            let y_coords = compute_y_coordinate(octant, origin, x, map,
                &mut top, &mut bottom);
            
                let top_y = y_coords.x;
                let bottom_y = y_coords.y;
    
            if !compute_visiblity(top_y, bottom_y, range, octant, origin, x, map, &mut top, &mut bottom) {
                break;
            }
            x += 1;
        }
    }
    
    fn compute_y_coordinate<T: VisiblityMap>(
        octant: i32, origin: IVec2, x: i32, map: &mut T,
        top: &mut Slope, bottom: &mut Slope 
    ) -> IVec2 {
        let mut top_y;
        if top.x == 1 {
            top_y = x;
        } else {
            top_y = ((x * 2 - 1) * top.y + top.x) / (top.x * 2);

            if blocks_light(x, top_y, octant, origin, map)
            {
                if top.greater_or_equal(top_y * 2 + 1, x * 2) && !blocks_light(x, top_y + 1, octant, origin, map) {
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

            if  bottom.greater_or_equal(bottom_y * 2 + 1, x * 2) 
            &&  blocks_light(x, bottom_y, octant, origin, map) 
            && !blocks_light(x, bottom_y + 1, octant, origin, map)
            {
                bottom_y += 1;
            }
        }
        IVec2::new(top_y, bottom_y)
    }
    
    fn compute_visiblity<T: VisiblityMap>(
        top_y: i32, bottom_y: i32,
        range: i32, octant: i32, origin: IVec2, x: i32, map: &mut T,
        top: &mut Slope, bottom: &mut Slope
    ) -> bool {
        let mut was_opaque = -1;
    
        for y in (bottom_y..=top_y).rev() {
            if range < 0 || map.dist(IVec2::ZERO, IVec2::new(x,y)) <= range as f32 {
                let is_opaque = blocks_light(x, y, octant, origin, map);

                // Less symmetrical
                // let is_visible = is_opaque ||
                // (
                //     (y != top_y || top.greater(y * 4 - 1, x * 4 + 1)) &&
                //     (y != bottom_y || bottom.less(y * 4 + 1, x * 4 - 1))
                // );
                
                // Better symmetry
                let is_visible = 
                is_opaque || // Uncomment for full symmetry but more artifacts in hallways 
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
                                    *bottom = Slope{ y: ny, x: nx };
                                    break;
                                }
                                else {
                                    compute_octant(octant, origin, range, x + 1, top.clone(), Slope { y: ny, x: nx }, map);
                                }
                            } else {
                                if y == bottom_y {
                                    return false;
                                }
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
    
        !(was_opaque != 0)
    }
    
    fn blocks_light<T: VisiblityMap>(x: i32, y: i32, octant: i32, origin: IVec2, map: &mut T) -> bool {
        let (mut nx,mut ny) = origin.into();
        match octant {
            0 => { nx += x; ny -= y; },
            1 => { nx += y; ny -= x; },
            2 => { nx -= y; ny -= x; },
            3 => { nx -= x; ny -= y; },
            4 => { nx -= x; ny += y; },
            5 => { nx -= y; ny += x; },
            6 => { nx += y; ny += x; },
            7 => { nx += x; ny += y; },
            _ => {}
        }
        let p = IVec2::new(nx,ny);
        if !map.is_in_bounds(p) {
            return true;
        }
        map.is_opaque(IVec2::new(nx,ny))
    }
    
    fn set_visible<T: VisiblityMap>(x: i32, y: i32, octant: i32, origin: IVec2, map: &mut T) {
        let (mut nx,mut ny) = origin.into();
        match octant {
            0 => { nx += x; ny -= y; },
            1 => { nx += y; ny -= x; },
            2 => { nx -= y; ny -= x; },
            3 => { nx -= x; ny -= y; },
            4 => { nx -= x; ny += y; },
            5 => { nx -= y; ny += x; },
            6 => { nx += y; ny += x; },
            7 => { nx += x; ny += y; },
            _ => {}
        }
        let p = IVec2::new(nx,ny);
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
            self.y *x <= self.x *y
        } // this <= y/x
    }

}


#[cfg(test)]
mod test {
    use glam::{IVec2, Vec2};

    use crate::{VisiblityMap, fov};

    struct Map {
        visible_points: Vec<bool>,
        opaque_points: Vec<bool>,
        width: i32,
        height: i32,
    }

    impl Map {
        fn to_index(&self, p: IVec2) -> usize {
            (p.y * self.width + p.x) as usize
        }
        
        fn set_opaque(&mut self, x: i32, y: i32) {
            let i = self.to_index(IVec2::new(x,y));
            self.opaque_points[i] = true;
        }

        fn is_visible(&self, x: i32, y: i32) -> bool {
            let p = IVec2::new(x,y);
            self.visible_points[self.to_index(p)]
        }
    }

    impl VisiblityMap for Map {
        fn is_opaque(&self, p: IVec2) -> bool {
            self.opaque_points[self.to_index(p)]
        }

        fn is_in_bounds(&self, p: IVec2) -> bool {
            p.x >= 0 && p.x < self.width &&
            p.y >= 0 && p.y < self.height
        }

        fn set_visible(&mut self, p: IVec2) {
            let i = self.to_index(p);
            self.visible_points[i] = true;
        }

        fn dist(&self, a: IVec2, b: IVec2) -> f32 {
            Vec2::distance(a.as_f32(),b.as_f32())
        }
    }

    #[test]
    fn test_fov() {
        let origin = IVec2::ZERO;
        let w = 30;
        let h = 30;
        let mut map = Map {
            visible_points: vec![false; w * h],
            opaque_points: vec![false; w * h],
            width: w as i32,
            height: h as i32,
        };
        map.set_opaque(0, 1);
        map.set_opaque(1, 0);
        fov::compute(origin, 5, &mut map);

        assert!( map.is_visible(0, 0) );

        assert!( map.is_visible(0, 1) );
        assert!( !map.is_visible(0, 2) );
        
        assert!( map.is_visible(1, 0) );
        assert!( !map.is_visible(2, 0) );
    }
}