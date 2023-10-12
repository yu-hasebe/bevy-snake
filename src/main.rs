use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::{PrimaryWindow, WindowResolution};
use rand::prelude::*;
use std::time::Duration;

const WINDOW_WIDTH: f32 = 500.0;
const WINDOW_HEIGHT: f32 = 500.0;

const WIDTH_IN_TILE: i32 = 10;
const HEIGHT_IN_TILE: i32 = 10;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const FOOD_COLOR: Color = Color::rgb(0.9, 0.4, 0.0);

const SNAKE_HEAD_SIZE: Size = Size {
    width: 0.8,
    height: 0.8,
};
const SNAKE_SEGMENT_SIZE: Size = Size {
    width: 0.7,
    height: 0.7,
};
const FOOD_SIZE: Size = Size {
    width: 0.8,
    height: 0.8,
};

const SNAKE_HEAD_INITIAL_POSITION: Position = Position { x: 4, y: 4 };
const SNAKE_SEGMENT_INITIAL_POSITION: Position = Position { x: 4, y: 5 };
const SNAKE_INITIAL_LAST_TAIL_POSITION: Position = Position { x: 4, y: 6 };
const SNAKE_HEAD_INITIAL_DIRECTION: Direction = Direction::Down;

#[derive(Component, Default, PartialEq, Copy, Clone)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Food;

#[derive(Component)]
struct SnakeHead;
#[derive(Component)]
struct SnakeSegment;

#[derive(Resource, Default)]
struct Snake {
    head: Option<Entity>,
    segments: Vec<Entity>,
    direction: Direction,
    last_tail_position: Position,
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}
impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
        }
    }
}
impl Default for Direction {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Event)]
struct GameOverEvent;

#[derive(Event)]
struct GrowthEvent;

fn main() {
    App::new()
        .init_resource::<Snake>()
        .add_event::<GameOverEvent>()
        .add_event::<GrowthEvent>()
        .add_systems(Startup, (setup_camera, spawn_snake))
        .add_systems(Update, snake_direction_input.before(snake_movement))
        .add_systems(
            Update,
            (snake_movement, snake_eating, snake_growth)
                .chain()
                .run_if(on_timer(Duration::from_millis(150))),
        )
        .add_systems(Update, game_over.after(snake_movement))
        .add_systems(
            Update,
            spawn_food.run_if(on_timer(Duration::from_millis(1000))),
        )
        .add_systems(PostUpdate, (transform, scale))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                ..default()
            }),
            ..default()
        }))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(BACKGROUND_COLOR),
            ..default()
        },
        ..default()
    });
}

fn spawn_snake(mut commands: Commands, mut snake: ResMut<Snake>) {
    snake.head = Some(
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(SnakeHead)
            .insert(SNAKE_HEAD_INITIAL_POSITION)
            .insert(SNAKE_HEAD_SIZE)
            .id(),
    );
    snake.segments = vec![spawn_segment(commands, SNAKE_SEGMENT_INITIAL_POSITION)];
    snake.direction = SNAKE_HEAD_INITIAL_DIRECTION;
    snake.last_tail_position = SNAKE_INITIAL_LAST_TAIL_POSITION;
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(position)
        .insert(SNAKE_SEGMENT_SIZE)
        .id()
}

fn snake_direction_input(keyboard_input: Res<Input<KeyCode>>, mut snake: ResMut<Snake>) {
    let current_dir = snake.direction;
    let new_dir = if keyboard_input.pressed(KeyCode::Left) {
        Direction::Left
    } else if keyboard_input.pressed(KeyCode::Up) {
        Direction::Up
    } else if keyboard_input.pressed(KeyCode::Right) {
        Direction::Right
    } else if keyboard_input.pressed(KeyCode::Down) {
        Direction::Down
    } else {
        current_dir
    };

    if new_dir.opposite() != current_dir {
        snake.direction = new_dir;
    }
}

fn snake_movement(
    mut snake: ResMut<Snake>,
    mut positions: Query<&mut Position>,
    mut game_over_event_writer: EventWriter<GameOverEvent>,
) {
    let segment_positions = snake
        .segments
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .collect::<Vec<Position>>();
    let mut head_position = positions.get_mut(snake.head.unwrap()).unwrap();
    let last_head_position = head_position.clone();

    let direction = snake.direction;

    match direction {
        Direction::Left => head_position.x -= 1,
        Direction::Down => head_position.y -= 1,
        Direction::Right => head_position.x += 1,
        Direction::Up => head_position.y += 1,
    }
    if head_position.x < 0
        || head_position.x >= WIDTH_IN_TILE
        || head_position.y < 0
        || head_position.y >= HEIGHT_IN_TILE
    {
        game_over_event_writer.send(GameOverEvent);
    }

    if segment_positions.contains(&head_position) {
        game_over_event_writer.send(GameOverEvent);
    }

    segment_positions
        .iter()
        .zip(snake.segments.iter().skip(1))
        .for_each(|(position, segment)| {
            *positions.get_mut(*segment).unwrap() = *position;
        });
    let mut tail_position = positions.get_mut(snake.segments[0]).unwrap();
    snake.last_tail_position = tail_position.clone();
    *tail_position = last_head_position;
}

fn snake_eating(
    mut commands: Commands,
    snake: Res<Snake>,
    positions: Query<&Position>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    mut growth_event_writer: EventWriter<GrowthEvent>,
) {
    let Ok(head_position) = positions.get(snake.head.unwrap()) else {
        return;
    };
    for (entity, food_position) in food_positions.iter() {
        if head_position == food_position {
            commands.entity(entity).despawn();
            growth_event_writer.send(GrowthEvent);
        }
    }
}

fn snake_growth(
    commands: Commands,
    mut snake: ResMut<Snake>,
    mut growth_event_reader: EventReader<GrowthEvent>,
) {
    if growth_event_reader.iter().next().is_none() {
        return;
    }

    let last_tail_position = snake.last_tail_position;
    snake
        .segments
        .push(spawn_segment(commands, last_tail_position));
}

fn game_over(
    mut commands: Commands,
    snake: ResMut<Snake>,
    positions: Query<(Entity, &Position)>,
    mut game_over_event_reader: EventReader<GameOverEvent>,
) {
    if game_over_event_reader.iter().next().is_none() {
        return;
    }

    for (entity, _) in positions.iter() {
        commands.entity(entity).despawn();
    }
    spawn_snake(commands, snake);
}

fn spawn_food(mut commands: Commands, positions: Query<&Position>) {
    let mut new_position = Position {
        x: (random::<f32>() * WIDTH_IN_TILE as f32) as i32,
        y: (random::<f32>() * HEIGHT_IN_TILE as f32) as i32,
    };
    while positions.iter().any(|p| *p == new_position) {
        new_position = Position {
            x: (random::<f32>() * WIDTH_IN_TILE as f32) as i32,
            y: (random::<f32>() * HEIGHT_IN_TILE as f32) as i32,
        };
    }

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Food)
        .insert(new_position)
        .insert(FOOD_SIZE);
}

fn transform(
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let Ok(window) = primary_windows.get_single() else {
        return;
    };
    for (position, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(position.x as f32, window.width(), WIDTH_IN_TILE as f32),
            convert(position.y as f32, window.height(), HEIGHT_IN_TILE as f32),
            0.0,
        );
    }
}

fn scale(
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
) {
    let Ok(window) = primary_windows.get_single() else {
        return;
    };
    for (size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            size.width / WIDTH_IN_TILE as f32 * window.width() as f32,
            size.height / HEIGHT_IN_TILE as f32 * window.height() as f32,
            1.0,
        );
    }
}
