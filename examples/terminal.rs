use adam_fov_rs::*;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_ascii_terminal::*;
use rand::Rng;
use sark_grids::{BitGrid, SizedGrid};

#[derive(Resource)]
struct ViewRange(usize);

#[derive(Resource, Deref, DerefMut)]
struct Walls(BitGrid);

#[derive(Resource, Deref, DerefMut)]
pub struct Vision(BitGrid);

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
    let size = [35, 35];
    commands.spawn(Terminal::new(size));
    commands.spawn(TerminalCamera::new());

    let mut map = Walls(BitGrid::new(size));
    place_walls(&mut map.0);
    commands.insert_resource(map);

    commands.insert_resource(Vision(BitGrid::new(size)));
    commands.insert_resource(ViewRange(5));
}

fn place_walls(walls: &mut BitGrid) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(0..walls.width());
        let y = rng.gen_range(0..walls.height());
        walls.set([x, y], true); // true == wall
    }
}

fn toggle_walls(
    mut map: ResMut<Walls>,
    mouse: Res<ButtonInput<MouseButton>>,
    q_cam: Query<&TerminalCamera>,
    q_term: Query<&TerminalTransform>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(p) = q_cam.single().cursor_world_pos() {
            let Some(p) = q_term.single().world_to_tile(p) else {
                return;
            };
            if map.in_bounds(p) {
                map.toggle(p);
            }
        }
    }
}

fn update_view_range(mut view_range: ResMut<ViewRange>, mut scroll_event: EventReader<MouseWheel>) {
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
    q_cam: Query<&TerminalCamera>,
) {
    let cam = q_cam.single();
    vision.set_all(false);
    let Some(cursor) = cam.cursor_world_pos() else {
        return;
    };
    if vision.in_bounds(cursor.as_ivec2()) {
        let tile_blocks_vision = |p| walls.get(p);
        let set_visible = |p| vision.set(p, true);
        compute_fov(
            cursor.as_ivec2(),
            range.0,
            walls.size(),
            tile_blocks_vision,
            set_visible,
        );
    }
}

fn update_terminal_from_map(
    vision: Res<Vision>,
    walls: Res<Walls>,
    mut q_term: Query<&mut Terminal>,
) {
    let mut term = q_term.single_mut();

    term.clear();

    for x in 0..term.width() as i32 {
        for y in 0..term.height() as i32 {
            if vision.get([x, y]) {
                if walls.get([x, y]) {
                    term.put_char([x, y], '#').fg(color::GREEN);
                } else {
                    term.put_char([x, y], '.').fg(color::WHITE);
                }
            }
        }
    }

    term.put_string([0, 0], "Click to toggle wall".clear_colors());
    term.put_string([0, 1], "Scroll to change view range".clear_colors());
}
