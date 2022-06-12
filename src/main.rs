#![windows_subsystem = "windows"]
extern crate piston_window;
extern crate opengl_graphics;
extern crate rand;

use opengl_graphics::GlGraphics;
use piston_window::*;
use rand::Rng;

const X:usize = 1280;
const Y:usize = 720;

const XGAME:usize = 320;
const YGAME:usize = 180;

const DENSITY:usize = 4;
const MAX_BOT_NUMBER:usize = 100;
const CONQUERING_RADIUS:i32 = 20;
const RANDOM_COLOR:[[f32; 4];6] = [
    [1.0, 1.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
];

struct App {
    window:PistonWindow,
    gl:GlGraphics,
}
#[derive(Clone, PartialEq)]
struct Snake {
    body:Vec<(i32,i32)>,
    direction:Direction,
    new_direction:Direction,
    player:Player
}
struct Map {
    state: Vec<Vec<MapState>>,
    void_space: Vec<(i32, i32)>
}
#[derive(PartialEq)]
struct MapState {
    conquered: bool,
    snake: Option<Player>,
}
#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Clone, Copy, PartialEq)]
enum Player {
    Human,
    Bot,
}

fn initialize_window() -> App
{
    let opengl = OpenGL::V3_2;
    let window: PistonWindow = WindowSettings::new("Tron", [X as u32, Y as u32])
        .exit_on_esc(true)
        .resizable(false)
        .vsync(false)
        .build()
        .unwrap();
    let gl = GlGraphics::new(opengl);
    return App{window, gl};
}

fn initialize_snake() -> Snake {
    let mut body:Vec<(i32,i32)> = Vec::new(); 
    body.push((20, 20));
    body.push((21, 20));
    body.push((22, 20));
    body.push((23, 20));
    return Snake {body, direction:Direction::Right, new_direction:Direction::Right, player:Player::Human};

}
fn initialize_snake_bots(ref mut map: &mut Map) -> Vec<Snake>  {
    let mut snake_bots:Vec<Snake> = Vec::new();
    for _ in 0..MAX_BOT_NUMBER {
        snake_bots.push(initialize_snake_bot(map))
    }
    return snake_bots;
}
fn initialize_snake_bot(ref mut map: &mut Map) -> Snake {
    let mut body:Vec<(i32,i32)> = Vec::new();
    
    let mut rng = rand::thread_rng();
    if map.void_space.len() == 0 {return Snake {body, direction:Direction::Right, new_direction:Direction::Right, player:Player::Bot}} //TODO
    let begin_point_vec = rng.gen_range(0, map.void_space.len());
    let begin_point = map.void_space[begin_point_vec];
    let (x,y) = begin_point;
    if map.state[x as usize][y as usize].snake != None {
        return Snake {body, direction:Direction::Right, new_direction:Direction::Right, player:Player::Bot}
    }
    let (x,y) = (x as i32, y as i32);
    body.push((x, y));
    let (x,y) = (x as usize, y as usize);
    map.state[x][y].snake = Some(Player::Bot);
    return Snake {body, direction:Direction::Right, new_direction:Direction::Right, player:Player::Bot};

}

fn initialize_map(ref snake : &Snake) -> Map {
    let mut state:Vec<Vec<MapState>> = Vec::new();
    let mut void_space = Vec::new();
    for x in 0..XGAME {
        let mut state_x:Vec<MapState> = Vec::new();
        for y in 0..YGAME {
            state_x.push(MapState {
                conquered: false,
                snake: None,
            });
            void_space.push((x as i32, y as i32));
        }
        state.push(state_x);
    }
    for body_part in snake.body.iter() {
        let (x, y) = *body_part;
        state[x as usize][y as usize].snake = Some(Player::Human);
        for i in -CONQUERING_RADIUS..CONQUERING_RADIUS {
            for j in -CONQUERING_RADIUS..CONQUERING_RADIUS {
                let x = x as i32 + i;
                let y = y as i32 + j;
                if x >= 0 && x < XGAME as i32 && y >= 0 && y < YGAME as i32 {
                    state[x as usize][y as usize].conquered = true;
                }
            }
        }
        void_space = Vec::new();
        for x in 0..state.len() {
            for y in 0..state[x].len() {
                if state[x][y].conquered == false {
                    void_space.push((x as i32, y as i32));
                }
            }
        }
    }

    let map = Map {
        state,
        void_space,
    };
    return map;
}

fn game_loop(mut app: App, mut snake: Snake, mut map: Map, mut snake_bots: Vec<Snake>) {
    let mut reset = false;
    let mut win = None;
    let mut lose = None;
    let mut time = std::time::Instant::now();
    app.window.set_max_fps(std::u64::MAX);
    loop {
        if app.window.should_close() { break }
        if time.elapsed() > std::time::Duration::from_millis(20) {
            if reset == true {
                reset = false;
                snake = initialize_snake();
                map = initialize_map(&snake);
                snake_bots = initialize_snake_bots(&mut map);
                win = None;
                lose = None;
            }
            if win == None {
                if map.void_space.is_empty() {
                    win = Some(std::time::Instant::now());
                    for bot in snake_bots.iter(){
                        if !bot.body.is_empty() {
                            win = None;
                        }
                    }
                }
            }
            if win == None && lose == None {
                let result = update(&mut snake, &mut map);
                if result == 0 { snake_bots = initialize_snake_bots(&mut map); lose = Some(std::time::Instant::now()); }
                for i in 0..snake_bots.len() {
                    update(&mut snake_bots[i], &mut map);
                }
            }
            time = std::time::Instant::now();
        }
        if let Some(e) = app.window.next() {
            let mut rng = rand::thread_rng();
            if let Some(e) = e.render_args() {
                app.gl.draw(e.viewport(), |c, g| {
                    clear([0.0; 4], g);
                    let mut x:usize=0;
                    let mut y:usize=0;
                    for temp in map.state.iter() {
                        for state in temp.iter() {
                            let mut color = match state.conquered {
                                true => [1.0, 1.0, 1.0, 1.0],
                                false => [0.0, 0.0, 0.0, 1.0],
                            };
                            if let Some(player) = state.snake {
                                color = match player {
                                    Player::Human => if win == None { [0.0, 1.0, 0.0, 1.0] } else { RANDOM_COLOR[rng.gen_range(0, 6)] },
                                    Player::Bot =>  [1.0, 0.0, 0.0, 1.0]
                                }
                            }
                            rectangle(color,
                                    [(x*DENSITY) as f64, (y*DENSITY) as f64, DENSITY as f64, DENSITY as f64],
                                    c.transform, g);
                            y = y + 1;
                        }
                        y = 0;
                        x = x +1;
                    }                
                    rectangle([0.0, 0.0, 0.0, 1.0],
                            [0.0, 0.0, (XGAME*DENSITY) as f64, 1.0],
                            c.transform, g);
                    rectangle([0.0, 0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0, (YGAME*DENSITY) as f64],
                            c.transform, g);
                    rectangle([0.0, 0.0, 0.0, 1.0],
                            [(XGAME*DENSITY) as f64, 0.0, 1.0, (YGAME*DENSITY) as f64],
                            c.transform, g);
                    rectangle([0.0, 0.0, 0.0, 1.0],
                            [0.0, (YGAME*DENSITY) as f64, (XGAME*DENSITY) as f64, 1.0],
                            c.transform, g);                        
                })
            }
            if let Some(e) = e.press_args() {
                if let Some(time) = win {
                    match e {
                        Button::Keyboard(Key::Up) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        Button::Keyboard(Key::Down) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        Button::Keyboard(Key::Left) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        Button::Keyboard(Key::Right) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        _ => {},
                    }
                }
                if let Some(time) = lose {
                    match e {
                        Button::Keyboard(Key::Up) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        Button::Keyboard(Key::Down) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        Button::Keyboard(Key::Left) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        Button::Keyboard(Key::Right) => if time.elapsed() > std::time::Duration::from_millis(500) { reset = true },
                        _ => {},
                    }
                }
                match e {
                    Button::Keyboard(Key::Up) => snake.new_direction = Direction::Up,
                    Button::Keyboard(Key::Down) => snake.new_direction = Direction::Down,
                    Button::Keyboard(Key::Left) => snake.new_direction = Direction::Left,
                    Button::Keyboard(Key::Right) => snake.new_direction = Direction::Right,
                    _ => (),
                }
            }
        }
    }
}

fn update(ref mut snake: &mut Snake, map: &mut Map) -> u32 {    
    if snake.player==Player::Bot {
        let mut rng = rand::thread_rng();
        let change_direction = rng.gen_range(0, 6);
        if change_direction == 0 {
            let direction = rng.gen_range(0, 4);
            match direction {
                0 => snake.new_direction = Direction::Up,
                1 => snake.new_direction = Direction::Down,
                2 => snake.new_direction = Direction::Left,
                3 => snake.new_direction = Direction::Right,
                _ => panic!(),
            }
        }
    }
    match snake.new_direction {
        Direction::Up => if snake.direction == Direction::Down { snake.new_direction = Direction::Down },
        Direction::Down => if snake.direction == Direction::Up { snake.new_direction = Direction::Up },
        Direction::Left => if snake.direction == Direction::Right { snake.new_direction = Direction::Right },
        Direction::Right => if snake.direction == Direction::Left { snake.new_direction = Direction::Left },
    }
    snake.direction = snake.new_direction.clone();
    return move_snake(snake, map);
}

fn move_snake(snake:&mut Snake, map: &mut Map) -> u32 {    
    if snake.body.len() == 0 {return 0} 
    let mut snake_head = *snake.body.last().unwrap();

    match snake.direction {
        Direction::Up => snake_head.1 = snake_head.1 - 1,
        Direction::Down => snake_head.1 = snake_head.1 + 1,
        Direction::Left => snake_head.0 = snake_head.0 - 1,
        Direction::Right => snake_head.0 = snake_head.0 + 1,
    }
    // avoid out of bounds
    if snake_head.0 < 0 || snake_head.0 >= XGAME as i32 || snake_head.1 < 0 || snake_head.1 >= YGAME as i32 {
        if snake.player == Player::Bot {
            for body_part in snake.body.iter(){
                let(x_temp,y_temp) = *body_part;
                map.state[x_temp as usize][y_temp as usize].snake = None;
            }
        }
        if snake.player == Player::Human {
            return 0
        }
        else { 
            *snake = initialize_snake_bot(map);
            return 0
        }
    }
    let (x,y) = snake_head;
    match map.state[x as usize][y as usize].snake {
        None => {
            snake.body.push(snake_head);          
        },
        Some(_player) => {
            if snake.player == Player::Bot {
                for body_part in snake.body.iter(){
                    let(x_temp,y_temp) = *body_part;
                    map.state[x_temp as usize][y_temp as usize].snake = None;
                }
            }
            match snake.player {
                Player::Human => {            
                    return 0
                }
                Player::Bot => { 
                *snake = initialize_snake_bot(map);
                return 0
                }
            }
        }
    }
    map.state[x as usize][y as usize].snake = Some(snake.player.clone());
    let (x,y) = (x as i32, y as i32);
    if snake.player == Player::Human {
        for i in -CONQUERING_RADIUS..CONQUERING_RADIUS {
            for j in -CONQUERING_RADIUS..CONQUERING_RADIUS {
                let x = x + i;
                let y = y + j;
                if x >= 0 && x < XGAME as i32 && y >= 0 && y < YGAME as i32 {
                    map.state[x as usize][y as usize].conquered = true;
                }
            }
        }
        map.void_space = Vec::new();
        for x in 0..map.state.len() {
            for y in 0..map.state[x].len() {
                if map.state[x][y].conquered == false {
                    map.void_space.push((x as i32, y as i32));
                }
            }
        }
    }
    return 1;
}

fn main() {
    let app: App= initialize_window();
    let snake: Snake = initialize_snake();
    let mut map: Map = initialize_map(&snake);
    let snake_bots:Vec<Snake> = initialize_snake_bots(&mut map);
    game_loop(app, snake, map, snake_bots);
}