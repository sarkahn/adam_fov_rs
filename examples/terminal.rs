use adam_fov_rs::*;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_ascii_terminal::*;
use rand::Rng;

#[derive(Resource)]
struct ViewRange(usize);

#[derive(Clone)]
enum Tile {
    Floor,
    Wall,
}

#[derive(Resource, Deref, DerefMut)]
struct Walls(Vec<Tile>);

#[derive(Resource, Deref, DerefMut)]
pub struct Vision(Vec<bool>);

const WIDTH: usize = 35;
const HEIGHT: usize = 35;

fn index(p: IVec2) -> usize {
    p.y as usize * WIDTH + p.x as usize
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_walls,
                update_view_range,
                update_vision,
                update_terminal_from_map.run_if(resource_changed::<Vision>),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Terminal::new([WIDTH as u32, HEIGHT as u32]));
    commands.spawn(TerminalCamera::new());

    let mut map = Walls(vec![Tile::Floor; WIDTH * HEIGHT]);
    place_walls(map.0.as_mut_slice());
    commands.insert_resource(map);

    commands.insert_resource(Vision(vec![false; WIDTH * HEIGHT]));
    commands.insert_resource(ViewRange(5));
}

fn place_walls(walls: &mut [Tile]) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(0..WIDTH);
        let y = rng.gen_range(0..HEIGHT);
        let i = index(IVec2::new(x as i32, y as i32));
        walls[i] = Tile::Wall;
    }
}

fn toggle_walls(
    mut map: ResMut<Walls>,
    mouse: Res<ButtonInput<MouseButton>>,
    q_cam: Single<&TerminalCamera>,
    q_term: Single<&TerminalTransform>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(p) = q_cam.cursor_world_pos() {
            let Some(p) = q_term.world_to_tile(p) else {
                return;
            };
            let size = IVec2::new(WIDTH as i32, HEIGHT as i32);
            if p.cmpge(IVec2::ZERO).all() && p.cmplt(size).all() {
                let i = index(p);
                map.0[i] = match map.0[i] {
                    Tile::Floor => Tile::Wall,
                    Tile::Wall => Tile::Floor,
                };
            }
        }
    }
}

fn update_view_range(
    mut view_range: ResMut<ViewRange>,
    mut scroll_event: MessageReader<MouseWheel>,
) {
    for ev in scroll_event.read() {
        let delta = ev.y.round() as i32;

        if delta == 0 {
            return;
        }

        view_range.0 = (view_range.0 as i32 + delta).max(1) as usize;
    }
}

fn update_vision(
    mut vision: ResMut<Vision>,
    walls: Res<Walls>,
    range: Res<ViewRange>,
    q_cam: Single<&TerminalCamera>,
) {
    vision.0.fill(false);
    let Some(cursor) = q_cam.cursor_world_pos().map(|p| p.as_ivec2()) else {
        return;
    };
    let bounds = IVec2::new(WIDTH as i32, HEIGHT as i32);
    if cursor.cmpge(IVec2::ZERO).all() && cursor.cmplt(bounds).all() {
        let tile_blocks_vision = |p| matches!(walls[index(p)], Tile::Wall);
        let set_visible = |p| vision[index(p)] = true;

        compute_fov(
            cursor,
            range.0,
            bounds.as_uvec2(),
            tile_blocks_vision,
            set_visible,
        );
    }
}

fn update_terminal_from_map(
    vision: Res<Vision>,
    walls: Res<Walls>,
    mut term: Single<&mut Terminal>,
) {
    term.clear();

    for x in 0..term.width() as i32 {
        for y in 0..term.height() as i32 {
            let i = y as usize * WIDTH + x as usize;
            if vision[i] {
                let (c, col) = match walls[i] {
                    Tile::Wall => ('#', color::GREEN),
                    Tile::Floor => ('.', color::WHITE),
                };
                term.put_char([x, y], c).fg(col);
            }
        }
    }

    term.put_string([0, 0], "Click to toggle wall".clear_colors());
    term.put_string([0, 1], "Scroll to change view range".clear_colors());
}
