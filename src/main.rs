//! Demonstrates how to observe life-cycle triggers as well as define custom ones.

use bevy::{
    color::palettes::basic::PURPLE, input::common_conditions::input_just_pressed, prelude::*,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "四方诛杀 KIVA".into(),
                resolution: (1280., 720.).into(),
                resizable: false,
                decorations: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(CursorWorldPos(None))
        .insert_resource(MoveWorldPos(None))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // get_cursor_world_pos,
                // object_follow_cursor,
                update_move_word_pos.run_if(input_just_pressed(MouseButton::Right)),
                move_to_word_pos,
                // player_movement,
            ),
        )
        .run();
}

#[derive(Resource)]
struct CursorWorldPos(Option<Vec2>);

#[derive(Resource)]
struct MoveWorldPos(Option<Vec2>);

#[derive(Component)]
pub struct Player {}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: meshes.add(Rectangle::default()).into(),
    //         transform: Transform::default().with_scale(Vec3::splat(50.)),
    //         material: materials.add(Color::from(PURPLE)),
    //         ..default()
    //     },
    //     Player {},
    // ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("logo.png"),
            transform: Transform::from_xyz(-100., 100., 0.),
            ..default()
        },
        Player {},
    ));
}

/* 玩家键盘移动 */
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction += Vec3::new(-1.0, 0.0, 0.0);

            if direction.length() > 0.0 {
                direction = direction.normalize();
            }
            println!("key_font: {}", transform.translation);
            transform.translation += direction * 1000.0 * time.delta_seconds();
            println!("key_back: {}", transform.translation);
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            direction += Vec3::new(1.0, 0.0, 0.0);

            if direction.length() > 0.0 {
                direction = direction.normalize();
            }
            println!("key_font: {}", transform.translation);
            transform.translation += direction * 1000.0 * time.delta_seconds();
            println!("key_back: {}", transform.translation);
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            direction += Vec3::new(0.0, 1.0, 0.0);

            if direction.length() > 0.0 {
                direction = direction.normalize();
            }
            println!("key_font: {}", transform.translation);
            transform.translation += direction * 1000.0 * time.delta_seconds();
            println!("key_back: {}", transform.translation);
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            direction += Vec3::new(0.0, -1.0, 0.0);

            if direction.length() > 0.0 {
                direction = direction.normalize();
            }
            println!("key_font: {}", transform.translation);
            transform.translation += direction * 1000.0 * time.delta_seconds();
            println!("key_back: {}", transform.translation);
        }

        // if direction.length() > 0.0 {
        //     direction = direction.normalize();
        // }
        // println!("font: {}", transform.translation);

        // transform.translation += direction * 1000.0 * time.delta_seconds();
        // println!("back: {}", transform.translation);
    }
}

fn get_cursor_world_pos(
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    q_primary_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let primary_window = q_primary_window.single();
    let (main_camera, main_camera_transform) = q_camera.single();
    // Get the cursor position in the world
    cursor_world_pos.0 = primary_window
        .cursor_position()
        .and_then(|cursor_pos| main_camera.viewport_to_world_2d(main_camera_transform, cursor_pos));
}

fn update_move_word_pos(
    mut move_world_pos: ResMut<MoveWorldPos>,
    q_primary_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let primary_window = q_primary_window.single();
    let (main_camera, main_camera_transform) = q_camera.single();
    // Get the cursor position in the world
    move_world_pos.0 = primary_window
        .cursor_position()
        .and_then(|cursor_pos| main_camera.viewport_to_world_2d(main_camera_transform, cursor_pos));
    println!("move_world_pos: {:?}", move_world_pos.0);
}

fn object_follow_cursor(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_world_pos: Res<CursorWorldPos>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        if buttons.pressed(MouseButton::Right) {
            let Some(cursor_world_pos) = cursor_world_pos.0 else {
                return;
            };

            transform.translation = cursor_world_pos.extend(transform.translation.z);
        }
    }
}

fn move_to_word_pos(
    move_world_pos: Res<MoveWorldPos>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let Some(move_world_pos) = move_world_pos.0 else {
            return;
        };

        let dist = transform.translation.truncate().distance(move_world_pos);
        if dist > 5. {
            println!("dist: {}", dist);
            let direction = (move_world_pos - transform.translation.truncate())
                .extend(transform.translation.z)
                .normalize();
            transform.translation += direction * 1000. * time.delta_seconds();
        }
    }
}

fn move_to_word_pos_with_same_speed(
    move_world_pos: Res<MoveWorldPos>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let Some(move_world_pos) = move_world_pos.0 else {
            return;
        };

        let dist_per_frame = 1000. * time.delta_seconds();
        let dist = transform.translation.truncate().distance(move_world_pos);
        if dist > 5. {
            println!("dist: {}", dist);
            let direction = (move_world_pos - transform.translation.truncate())
                .extend(transform.translation.z)
                .normalize();
            transform.translation += direction * 1000. * time.delta_seconds();
        }
    }
}
