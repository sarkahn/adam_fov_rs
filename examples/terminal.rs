use adam_fov_rs::*;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_ascii_terminal::*;
use rand::Rng;
use sark_grids::SizedGrid;

#[derive(Resource)]
struct ViewRange(i32);

#[derive(Resource, Deref, DerefMut)]
pub struct Visibility(VisibilityMap);

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
                update_terminal_from_map.run_if(resource_changed::<Visibility>),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    let size = [35, 35];
    commands.spawn(Terminal::new(size));
    commands.spawn(TerminalCamera::new());

    let mut map = Visibility(VisibilityMap::new(size));
    place_walls(&mut map);
    commands.insert_resource(map);

    commands.insert_resource(ViewRange(5));
}

fn place_walls(map: &mut VisibilityMap) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(0..map.width());
        let y = rng.gen_range(0..map.height());
        map.add_blocker([x, y]);
    }
}

fn toggle_walls(
    mut map: ResMut<Visibility>,
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
                map.toggle_blocker(p);
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

        view_range.0 += delta;
    }
}

fn update_vision(
    mut visibility: ResMut<Visibility>,
    range: Res<ViewRange>,
    q_cam: Query<&TerminalCamera>,
) {
    let cam = q_cam.single();
    visibility.clear_visibility();
    let Some(cursor) = cam.cursor_world_pos() else {
        return;
    };
    if visibility.in_bounds(cursor.as_ivec2()) {
        visibility.compute(cursor.as_ivec2(), range.0);
    }
}

fn update_terminal_from_map(map: Res<Visibility>, mut q_term: Query<&mut Terminal>) {
    let mut term = q_term.single_mut();

    term.clear();

    for x in 0..term.width() as i32 {
        for y in 0..term.height() as i32 {
            if map.is_visible([x, y]) {
                if map.is_blocker([x, y]) {
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
