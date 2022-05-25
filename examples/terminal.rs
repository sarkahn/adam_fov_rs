use adam_fov_rs::*;
use bevy::{input::mouse::MouseWheel, prelude::*, render::camera::Camera};
use bevy_ascii_terminal::*;
use bevy_tiled_camera::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TiledCameraPlugin)
        .add_plugin(TerminalPlugin)
        .add_startup_system(setup)
        .add_system(toggle_walls)
        .add_system(update_cursor_pos)
        .add_system(update_view_range)
        .add_system(update_terminal_from_map)
        .run();
}

fn setup(mut commands: Commands) {
    let size = [35, 35];
    commands.spawn_bundle(TerminalBundle::new().with_size(size));

    commands.spawn_bundle(TiledCameraBundle::new().with_tile_count(size));

    let mut map = Map::new(size);
    place_walls(&mut map);
    commands.insert_resource(map);

    commands.insert_resource(CursorPos::default());
    commands.insert_resource(ViewRange(5));
}

fn place_walls(map: &mut Map) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(0..map.width);
        let y = rng.gen_range(0..map.height);
        map.toggle_opaque(IVec2::new(x, y));
    }
}

fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    windows: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    mut map: ResMut<Map>,
    view_range: Res<ViewRange>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(pos) = window.cursor_position() {
        let (cam, t, proj) = q_cam.single();

        if let Some(pos) = proj.screen_to_world(cam, &windows, t, pos) {
            let pos = pos.truncate().round().as_ivec2();
            // println!("Cursor world position: {}", pos);
            if cursor_pos.0 != pos || view_range.is_changed() {
                cursor_pos.0 = pos;
                map.clear_visible();
                let pos = world_to_map(&map, pos);
                fov::compute(pos, view_range.0, &mut *map);
            }
        }
    }
}

fn toggle_walls(mut map: ResMut<Map>, cursor_pos: Res<CursorPos>, mouse: Res<Input<MouseButton>>) {
    if mouse.just_pressed(MouseButton::Left) {
        let p = cursor_pos.0;
        let p = world_to_map(&map, p);
        if map.is_in_bounds(p) {
            map.toggle_opaque(p);
        }
    }
}

struct ViewRange(i32);
fn update_view_range(mut view_range: ResMut<ViewRange>, mut scroll_event: EventReader<MouseWheel>) {
    for ev in scroll_event.iter() {
        let delta = ev.y.ceil() as i32;

        if delta == 0 {
            return;
        }

        view_range.0 += delta;
    }
}

fn update_terminal_from_map(map: Res<Map>, mut q_term: Query<&mut Terminal>) {
    if map.is_changed() {
        let mut term = q_term.single_mut();

        term.clear();

        let wall = CharFormat::new(Color::GREEN, Color::BLACK);
        let floor = CharFormat::new(Color::WHITE, Color::BLACK); 
        for x in 0..term.width() as i32 {
            for y in 0..term.height() as i32 {
                if map.is_visible(x, y) {
                    let p = IVec2::new(x, y);
                    if map.is_opaque(p) {
                        //let y = (term.height() - 1) as i32 - y;
                        term.put_char_formatted([x, y], '#', wall);
                    } else {
                        //let y = (term.height() - 1) as i32 - y;
                        term.put_char_formatted([x, y], '.', floor);
                    }
                }
            }
        }

        term.put_string([0, 0], "Click to toggle wall");
        term.put_string([0, 1], "Scroll to change view range");
    }
}

fn world_to_map(map: &Map, mut world: IVec2) -> IVec2 {
    let half_w = map.width / 2;
    let half_h = map.height / 2;
    world.x += half_w;
    world.y += half_h;
    world
}

struct Map {
    visible_points: Vec<bool>,
    opaque_points: Vec<bool>,
    width: i32,
    height: i32,
}

impl Map {
    fn new(size: [u32;2]) -> Self {
        let [w, h] = size;
        let len = (w * h) as usize;
        Map {
            visible_points: vec![false; len],
            opaque_points: vec![false; len],
            width: w as i32,
            height: h as i32,
        }
    }

    fn to_index(&self, p: IVec2) -> usize {
        (p.y * self.width + p.x) as usize
    }

    fn is_visible(&self, x: i32, y: i32) -> bool {
        let p = IVec2::new(x, y);
        if !self.is_in_bounds(p) {
            return false;
        }
        self.visible_points[self.to_index(p)]
    }

    fn toggle_opaque(&mut self, p: IVec2) {
        let i = self.to_index(p);
        self.opaque_points[i] = !self.opaque_points[i];
    }

    fn clear_visible(&mut self) {
        let len = self.width * self.height;
        self.visible_points = vec![false; len as usize];
    }
}

impl VisibilityMap for Map {
    fn is_opaque(&self, p: IVec2) -> bool {
        if !self.is_in_bounds(p) {
            return true;
        }
        self.opaque_points[self.to_index(p)]
    }

    fn is_in_bounds(&self, p: IVec2) -> bool {
        p.x >= 0 && p.x < self.width && p.y >= 0 && p.y < self.height
    }

    fn set_visible(&mut self, p: IVec2) {
        if !self.is_in_bounds(p) {
            return;
        }
        let i = self.to_index(p);
        self.visible_points[i] = true;
    }

    fn dist(&self, a: IVec2, b: IVec2) -> f32 {
        Vec2::distance(a.as_vec2(), b.as_vec2())
    }
}

#[derive(Default)]
struct CursorPos(IVec2);

