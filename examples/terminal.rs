use adam_fov_rs::*;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_ascii_terminal::{prelude::*, ToWorld};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
    commands
        .spawn_bundle(TerminalBundle::new().with_size(size))
        .insert(AutoCamera)
        .insert(ToWorld::default());

    let mut map = VisibilityMap2d::default(size);
    place_walls(&mut map);
    commands.insert_resource(map);

    commands.insert_resource(CursorPos::default());
    commands.insert_resource(ViewRange(5));
}

fn place_walls(map: &mut VisibilityMap2d) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(0..map.width());
        let y = rng.gen_range(0..map.height());
        map[[x, y]].opaque = true;
    }
}

fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    windows: Res<Windows>,
    mut map: ResMut<VisibilityMap2d>,
    view_range: Res<ViewRange>,
    q_tw: Query<&ToWorld>,
) {
    if let Some(window) = windows.get_primary() {
        if let Some(pos) = window.cursor_position() {
            if let Ok(tw) = q_tw.get_single() {
                if let Some(pos) = tw.screen_to_world(pos) {
                    let pos = pos.round().as_ivec2();
                    if cursor_pos.0 != pos || view_range.is_changed() {
                        cursor_pos.0 = pos;
                        map.clear_visible();
                        let pos = map.world_to_grid(pos);
                        fov::compute(pos, view_range.0, &mut *map);
                    }
                }
            }
        }
    }
}

fn toggle_walls(
    mut map: ResMut<VisibilityMap2d>,
    cursor_pos: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let p = cursor_pos.0;
        let p = map.world_to_grid(p);
        if map.is_in_bounds(p) {
            let p = &mut map[p].opaque;
            *p = !*p;
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

fn update_terminal_from_map(map: Res<VisibilityMap2d>, mut q_term: Query<&mut Terminal>) {
    if map.is_changed() {
        let mut term = q_term.single_mut();

        term.clear();

        for x in 0..term.width() as i32 {
            for y in 0..term.height() as i32 {
                if map[[x, y]].visible {
                    if map[[x, y]].opaque {
                        term.put_char([x, y], '#'.fg(Color::GREEN));
                    } else {
                        term.put_char([x, y], '.'.fg(Color::WHITE));
                    }
                }
            }
        }

        term.put_string([0, 0], "Click to toggle wall");
        term.put_string([0, 1], "Scroll to change view range");
    }
}

#[derive(Default)]
struct CursorPos(IVec2);
