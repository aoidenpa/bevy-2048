#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]
use std::time::Duration;

use crate::ui::GameUiPlugin;
use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    utils::HashMap,
    window::{PresentMode, WindowResolution},
};
use bevy_aseprite_ultra::prelude::*;
use bevy_tweening::*;
use lens::TransformPositionLens;
use rand::{thread_rng, Rng};

mod ui;
fn main() {
    let config = Config::new();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(config.window_size.x, config.window_size.y)
                        .with_scale_factor_override(1.0),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(BevySprityPlugin)
    .add_plugins(TweeningPlugin)
    .add_plugins(GameUiPlugin)
    .insert_resource(config)
    .insert_state(AppState::Setup)
    .insert_resource(ClearColor(Color::linear_rgb(1.0, 1.0, 1.0)))
    .add_event::<AddPieceEvent>()
    .add_event::<SetValueEvent>()
    .add_event::<MoveEvent>()
    .add_event::<NewGameEvent>()
    .add_systems(
        OnEnter(AppState::Setup),
        (
            setup,
            apply_deferred,
            create_board,
            apply_deferred,
            start_game,
        )
            .in_set(InitSet)
            .chain(),
    )
    .add_systems(Update, input)
    .add_systems(Update, (check_anim_end).run_if(in_state(AppState::Anim)))
    .add_systems(
        Update,
        (anim_completed_event).run_if(on_event::<TweenCompleted>()),
    )
    .add_systems(Update, (new_game_event).run_if(on_event::<NewGameEvent>()))
    .add_systems(
        Update,
        (set_board, process_move)
            .chain()
            .run_if(on_event::<MoveEvent>()),
    )
    .add_systems(
        Update,
        (set_value_event)
            .chain()
            .run_if(on_event::<SetValueEvent>()),
    )
    .add_systems(OnEnter(AppState::PostAnim), (set_board, post_anim).chain())
    //.add_systems(OnEnter(AppState::Input), check_game_end)
    .add_systems(
        Update,
        (add_piece_event, apply_deferred, check_game_end)
            .chain()
            .run_if(on_event::<AddPieceEvent>()),
    );
    app.run();
}
fn setup(
    mut commands: Commands,
    mut add_event: EventWriter<AddPieceEvent>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
) {
    commands.spawn((Camera2dBundle {
        transform: Transform::from_xyz(config.window_size.x, -config.window_size.y, 100.0),
        projection: OrthographicProjection { ..default() },
        ..default()
    },));
    add_event.send(AddPieceEvent(2));
    let font = asset_server.load("mai10/mai10.ttf");
    let title_font = asset_server.load("Early GameBoy.ttf");
    let sprite = asset_server.load("sprites.aseprite");
    commands.insert_resource(PieceFont(font));
    commands.insert_resource(TitleFont(title_font));
    commands.insert_resource(SpriteHandle(sprite));
    commands.insert_resource(Board {
        pieces: vec![None; (config.size * config.size) as usize],
    });
    let pos = config.window_size - Vec2::new(config.board_size(), config.board_size()) / 2.0;
    let pivot = commands
        .spawn((
            TransformBundle {
                local: Transform::from_xyz(pos.x, -pos.y, 0.0),
                ..default()
            },
            VisibilityBundle::default(),
        ))
        .id();
    commands.insert_resource(BoardPivot(pivot));
    let mut map = HashMap::new();
    map.insert(2, Color::srgb_u8(255, 209, 0));
    map.insert(4, Color::srgb_u8(255, 132, 38));
    map.insert(8, Color::srgb_u8(214, 36, 17));
    map.insert(16, Color::srgb_u8(255, 128, 164));
    map.insert(32, Color::srgb_u8(255, 38, 116));
    map.insert(64, Color::srgb_u8(191, 255, 60));
    map.insert(128, Color::srgb_u8(16, 210, 117));
    map.insert(256, Color::srgb_u8(40, 200, 225));
    map.insert(512, Color::srgb_u8(31, 85, 148));
    map.insert(1024, Color::srgb_u8(67, 0, 103));
    map.insert(2048, Color::srgb_u8(148, 33, 106));
    map.insert(4096, Color::srgb_u8(155, 124, 68));
    map.insert(8192, Color::srgb_u8(199, 113, 244));
    map.insert(16384, Color::srgb_u8(103, 40, 225));
    map.insert(32768, Color::srgb_u8(237, 199, 131));
    map.insert(65536, Color::srgb_u8(144, 52, 192));

    commands.insert_resource(ColorMap { map });
    commands.insert_resource(Score(0));
    commands.insert_resource(ScoreToAdd(0));
    commands.insert_resource(HighScore(0));
}
fn process_move(
    mut commands: Commands,
    mut move_event: EventReader<MoveEvent>,
    mut board: ResMut<Board>,
    config: Res<Config>,
    mut next_state: ResMut<NextState<AppState>>,
    mut score_to_add: ResMut<ScoreToAdd>,
) {
    for event in move_event.read() {
        let dir;
        let start_dir;
        let start;
        let size = config.size;
        match event {
            MoveEvent::Up => {
                dir = (0, 1);
                start = (0, 0);
                start_dir = (1, 0);
            }
            MoveEvent::Down => {
                dir = (0, -1);
                start = (0, size - 1);
                start_dir = (1, 0);
            }
            MoveEvent::Left => {
                dir = (1, 0);
                start = (0, 0);
                start_dir = (0, 1);
            }
            MoveEvent::Right => {
                dir = (-1, 0);
                start = (size - 1, 0);
                start_dir = (0, 1);
            }
        }
        let mut cur = start;
        for _ in 0..size {
            update_line(
                &mut commands,
                &mut board,
                cur.into(),
                dir.into(),
                &config,
                &mut score_to_add,
            );
            cur.0 += start_dir.0;
            cur.1 += start_dir.1;
        }
        next_state.set(AppState::Anim);
    }
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut move_event: EventWriter<MoveEvent>,
    mut new_game: EventWriter<NewGameEvent>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Input {
        if keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp) {
            move_event.send(MoveEvent::Up);
        } else if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
            move_event.send(MoveEvent::Left);
        } else if keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown) {
            move_event.send(MoveEvent::Down);
        } else if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
            move_event.send(MoveEvent::Right);
        }
    }
    if *state.get() == AppState::Input || *state.get() == AppState::GameOver {
        if keys.just_pressed(KeyCode::KeyR) {
            new_game.send(NewGameEvent);
        }
    }
}
fn update_line(
    commands: &mut Commands,
    board: &mut Board,
    start: IVec2,
    dir: IVec2,
    config: &Config,
    score_to_add: &mut ScoreToAdd,
) {
    let size = config.size;
    let mut cur = start;
    let mut stack: Vec<(Piece, bool)> = vec![];
    for _ in 0..size {
        let stack_len = stack.len();
        if let Some(piece) = board.pieces[to_index(cur, size) as usize] {
            let pos;
            if let Some(last) = stack.last_mut() {
                if last.0.value == piece.value && !last.1 {
                    last.0.value *= 2;
                    last.1 = true;
                    pos = start + dir * (stack_len as i32 - 1);
                    commands
                        .entity(piece.entity)
                        .insert(MoveType::MoveDouble((pos, last.0.entity)));
                    score_to_add.0 += last.0.value;
                } else {
                    pos = start + dir * (stack.len() as i32);
                    commands.entity(piece.entity).insert(MoveType::Move(pos));
                    stack.push((piece, false));
                }
            } else {
                pos = start;
                commands.entity(piece.entity).insert(MoveType::Move(start));
                stack.push((piece, false));
            }
            let start = pos_to_world(piece.pos, config);
            let end = pos_to_world(pos, config);
            let d = (end - start).length();
            let t = d / 4000.0;
            let tween = Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(t + 0.01),
                TransformPositionLens {
                    start: start.extend(2.0),
                    end: end.extend(2.0),
                },
            )
            .with_completed_event(0);
            commands.entity(piece.entity).insert(Animator::new(tween));
        }
        cur += dir;
    }
}

fn pos_to_world(pos: IVec2, config: &Config) -> Vec2 {
    (
        (pos.x * config.tile_size + config.tile_size / 2 + config.pad + 2 * pos.x * config.pad)
            as f32,
        -(pos.y * config.tile_size + config.tile_size / 2 + config.pad + 2 * pos.y * config.pad)
            as f32,
    )
        .into()
}
fn add_piece_event(
    mut commands: Commands,
    mut add_event: EventReader<AddPieceEvent>,
    mut board: ResMut<Board>,
    config: Res<Config>,
    font: Res<PieceFont>,
    sprite: Res<SpriteHandle>,
    pivot: Res<BoardPivot>,
    color_map: Res<ColorMap>,
) {
    let mut empties = vec![];
    for i in 0..board.pieces.len() {
        if board.pieces[i].is_none() {
            empties.push(i);
        }
    }
    if empties.is_empty() {
        return;
    }
    let mut rng = thread_rng();
    for event in add_event.read() {
        for _ in 0..event.0 {
            let pos = empties.remove(rng.gen_range(0..empties.len()));
            let value = if rng.gen_range(0..10) == 0 { 4 } else { 2 };
            create_piece(
                &mut commands,
                to_pos(pos as i32, config.size),
                value,
                &config,
                font.0.clone_weak(),
                sprite.0.clone_weak(),
                &pivot,
                &color_map,
                &mut board,
            );
        }
    }
}

fn to_index(pos: IVec2, size: i32) -> i32 {
    pos.y * size + pos.x
}
fn to_pos(index: i32, size: i32) -> IVec2 {
    (index % size, index / size).into()
}
fn create_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
    pivot: Res<BoardPivot>,
) {
    for i in 0..config.size {
        for j in 0..config.size {
            let world_pos = pos_to_world((i, j).into(), &config);
            commands
                .spawn(AsepriteSliceBundle {
                    slice: "back".into(),
                    aseprite: asset_server.load("sprites.aseprite"),
                    transform: Transform::from_xyz(world_pos.x, world_pos.y, 2.0),
                    ..default()
                })
                .set_parent(pivot.0);
        }
    }
}

fn create_piece(
    commands: &mut Commands,
    pos: IVec2,
    value: i32,
    config: &Config,
    font: Handle<Font>,
    sprite: Handle<Aseprite>,
    pivot: &BoardPivot,
    color_map: &ColorMap,
    board: &mut Board,
) {
    let color = match color_map.map.get(&value) {
        Some(c) => *c,
        None => Color::BLACK,
    };
    let base = commands
        .spawn(AsepriteSliceBundle {
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            slice: "piece".into(),
            aseprite: sprite,
            sprite: Sprite { color, ..default() },
            ..default()
        })
        .id();
    let text = commands
        .spawn(Text2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 4.0),
            text: Text::from_section(
                value.to_string(),
                TextStyle {
                    font,
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            ),
            ..default()
        })
        .id();
    let world_pos = pos_to_world(pos, config);
    let piece = commands
        .spawn((
            PieceMarker,
            TransformBundle {
                local: Transform::from_xyz(world_pos.x, world_pos.y, 2.0),
                ..default()
            },
            VisibilityBundle { ..default() },
            Value(value),
            Pos(pos),
        ))
        .push_children(&[base, text])
        .set_parent(pivot.0)
        .id();
    board.pieces[to_index(pos, config.size) as usize] = Some(Piece {
        entity: piece,
        value,
        pos,
    });
}

fn set_board(query: Query<(Entity, &Value, &Pos)>, mut board: ResMut<Board>, config: Res<Config>) {
    for i in 0..config.size * config.size {
        board.pieces[i as usize] = None;
    }

    for (entity, value, pos) in query.iter() {
        board.pieces[to_index(pos.0, config.size) as usize] = Some(Piece {
            entity,
            value: value.0,
            pos: pos.0,
        });
    }
}
fn check_anim_end(
    query: Query<Entity, With<MoveType>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if query.is_empty() {
        next_state.set(AppState::PostAnim);
    }
}

fn anim_completed_event(
    mut commands: Commands,
    mut anim_event: EventReader<TweenCompleted>,
    query: Query<(Entity, &MoveType), With<PieceMarker>>,
    mut pos_query: Query<&mut Pos>,
    mut value_query: Query<&mut Value>,
    mut set_value_event: EventWriter<SetValueEvent>,
) {
    for event in anim_event.read() {
        if let Ok((entity, move_type)) = query.get(event.entity) {
            match move_type {
                MoveType::Move(pos) => {
                    if let Ok(mut cur_pos) = pos_query.get_mut(entity) {
                        *cur_pos = Pos(*pos);
                    }
                    commands.entity(entity).remove::<MoveType>();
                }
                MoveType::MoveDouble((_pos, target)) => {
                    if let Ok(mut target_value) = value_query.get_mut(*target) {
                        target_value.0 *= 2;
                        set_value_event.send(SetValueEvent {
                            entity: *target,
                            value: target_value.0,
                        });
                    }
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

fn set_value_event(
    mut set_value_event: EventReader<SetValueEvent>,
    query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    mut sprite_query: Query<&mut Sprite>,
    font: Res<PieceFont>,
    color_map: Res<ColorMap>,
) {
    for event in set_value_event.read() {
        if let Ok(children) = query.get(event.entity) {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    *text = Text::from_section(
                        event.value.to_string(),
                        TextStyle {
                            font: font.0.clone_weak(),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                    );
                }
                if let Ok(mut sprite) = sprite_query.get_mut(*child) {
                    sprite.color = color_map.map[&event.value];
                }
            }
        }
    }
}

fn post_anim(
    mut add_event: EventWriter<AddPieceEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut score: ResMut<Score>,
    mut score_to_add: ResMut<ScoreToAdd>,
    mut high_score: ResMut<HighScore>,
) {
    add_event.send(AddPieceEvent(1));
    next_state.set(AppState::Input);
    score.0 += score_to_add.0;
    score_to_add.0 = 0;
    high_score.0 = high_score.0.max(score.0);
}

fn new_game_event(
    mut commands: Commands,
    new_game_event: EventReader<NewGameEvent>,
    mut board: ResMut<Board>,
    mut add_event: EventWriter<AddPieceEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    query: Query<Entity, With<PieceMarker>>,
    mut score: ResMut<Score>,
) {
    if !new_game_event.is_empty() {
        board.pieces.fill(None);
        add_event.send(AddPieceEvent(2));
        next_state.set(AppState::Input);
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
    score.0 = 0;
}

fn check_game_end(
    board: Res<Board>,
    mut next_state: ResMut<NextState<AppState>>,
    config: Res<Config>,
) {
    for piece in &board.pieces {
        if piece.is_none() {
            return;
        }
    }
    let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
    for i in 0..config.size {
        for j in 0..config.size {
            let cur = (i, j);
            for dir in dirs {
                let neigh = (cur.0 + dir.0, cur.1 + dir.1);
                if neigh.0 >= 0 && neigh.0 < config.size && neigh.1 >= 0 && neigh.1 < config.size {
                    if let Some(p1) = board.pieces[to_index(cur.into(), config.size) as usize] {
                        if let Some(p2) = board.pieces[to_index(neigh.into(), config.size) as usize]
                        {
                            if p1.value == p2.value {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
    next_state.set(AppState::GameOver);
}

fn start_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Input);
}

#[derive(Component)]
struct PieceMarker;

#[derive(Event)]
enum MoveEvent {
    Up,
    Down,
    Right,
    Left,
}

#[derive(Component)]
enum MoveType {
    Move(IVec2),
    MoveDouble((IVec2, Entity)),
}

#[derive(Clone, Copy)]
struct Piece {
    entity: Entity,
    value: i32,
    pos: IVec2,
}
#[derive(Resource)]
struct Board {
    pieces: Vec<Option<Piece>>,
}
#[derive(Component)]
struct Value(i32);
#[derive(Component)]
struct Pos(IVec2);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Setup,
    Input,
    Anim,
    PostAnim,
    GameOver,
}

#[derive(Resource)]
struct Config {
    size: i32,
    tile_size: i32,
    pad: i32,
    window_size: Vec2,
}
impl Config {
    fn new() -> Self {
        Self {
            size: 4,
            tile_size: 150,
            pad: 0,
            window_size: (900.0, 900.0).into(),
        }
    }
    fn board_size(&self) -> f32 {
        (self.size * (self.tile_size + 2 * self.pad)) as f32
    }
}
#[derive(Event)]
struct AddPieceEvent(i32);

#[derive(Event)]
struct SetValueEvent {
    entity: Entity,
    value: i32,
}

#[derive(Resource)]
struct PieceFont(Handle<Font>);
#[derive(Resource)]
struct TitleFont(Handle<Font>);
#[derive(Resource)]
struct SpriteHandle(Handle<Aseprite>);

#[derive(Resource)]
struct ColorMap {
    map: HashMap<i32, Color>,
}

#[derive(Resource)]
struct BoardPivot(Entity);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InitSet;

#[derive(Event)]
struct NewGameEvent;

#[derive(Resource)]
struct Score(i32);
#[derive(Resource)]
struct ScoreToAdd(i32);
#[derive(Resource)]
struct HighScore(i32);
