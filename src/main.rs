use rand::{seq::SliceRandom, Rng, distributions::uniform::{SampleUniform, SampleBorrow}};
use std::thread;
use std::time::{Duration, Instant};
use std::ops::{Add, Sub};
use std::cmp::{max, min};
use termbuffer::{char, App, Color, Draw, Event, Key};
use num_traits::Num;

//const FRAME_TIME: Duration = Duration::from_millis(1000 / 60); // 60 fps
const FRAME_TIME: Duration = Duration::from_millis(1000 / 20);
//const FRAME_TIME: Duration = Duration::from_millis(1000);
const MAX_ROOM_SIZE: isize = 15;
const MIN_ROOM_SIZE: isize = 4;

fn print_panic(info: &std::panic::PanicInfo) {
    use std::io::Write;
    let bt = backtrace::Backtrace::new();
    let mut panic_file = std::fs::File::create("panic_report.log").unwrap();
    write!(panic_file, "{:#?}", info).unwrap();
    write!(panic_file, "{:#?}", bt).unwrap();
}

fn main() {
    std::panic::set_hook(Box::new(print_panic));

    let mut shutdown = false;

    let mut world = World::gen(100, 100, &mut rand::thread_rng());
    //println!("{:#?}", world);
    //return;
    let mut app = App::builder().build().unwrap();
    loop {
        let time_start = Instant::now();
        {
            let mut draw = app.draw();
            world.draw(&mut draw);
        }
        for evt in app.events() {
            match evt.unwrap() {
                Event::Key(Key::Char('q')) => {
                    shutdown = true;
                }
                Event::Key(Key::Up) => {
                    world.player.move_by(Direction::Up, &world.rooms)
                }
                Event::Key(Key::Down) => {
                    world.player.move_by(Direction::Down, &world.rooms)
                }
                Event::Key(Key::Left) => {
                    world.player.move_by(Direction::Left, &world.rooms)
                }
                Event::Key(Key::Right) => {
                    world.player.move_by(Direction::Right, &world.rooms)
                }



                _ => (),
            }
        }
        if shutdown {
            break;
        }
        let time_end = Instant::now();
        if time_end < time_start + FRAME_TIME {
            thread::sleep(FRAME_TIME - (time_end - time_start));
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct World {
    width: isize,
    height: isize,
    player: Player,
    rooms: Vec<Room>,
}

impl World {
    pub fn gen<R: Rng + ?Sized>(width: isize, height: isize, rng: &mut R) -> World {
        let num_rooms = (width * height) / 300;
        let mut rooms = vec![];
        for _ in 0..num_rooms {
            let room_width = rng.gen_range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let room_height = rng.gen_range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let left = rng.gen_range(0, width - room_width);
            let top = rng.gen_range(0, height - room_height);
            let room_rect = Rectangle::new(
                Position { x: left, y: top },
                Position {
                    x: left + room_width,
                    y: top + room_height,
                },
            );
            rooms.push(Room::new(room_rect));
        }
        let player = Player::new_unchecked(rooms
            .choose(rng)
            .expect("no rooms")
            .random_location(rng));
        World { width, height, player, rooms, }
    }

    pub fn draw(&self, draw: &mut Draw) {
        let screen_width = draw.columns() as isize;
        let screen_height = draw.rows() as isize;
        // Left and top are limited by boundary
        let screen_left = self.player.x() - screen_width / 2;
        let screen_top = self.player.y() - screen_height / 2;
        let screen_right = screen_left + screen_width;
        let screen_bottom = screen_top + screen_height;
        let screen = Rectangle::new(
            Position {
                x: screen_left,
                y: screen_top,
            },
            Position {
                x: screen_right,
                y: screen_bottom,
            },
        );
        for room in self.rooms.iter() {
            room.draw(screen, draw);
        }
        self.player.draw(screen, draw);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Room {
    bounds: Rectangle<isize>,
}

impl Room {
    pub fn new(bounds: Rectangle<isize>) -> Self {
        Room { bounds }
    }

    pub fn contains(&self, point: Position<isize>) -> bool {
        point.x > self.bounds.top_left.x && point.x < self.bounds.bottom_right.x - 1
            && point.y > self.bounds.top_left.y && point.y < self.bounds.bottom_right.y - 1
    }

    pub fn draw(&self, screen: Rectangle<isize>, draw: &mut Draw) {
        let bounds = self.bounds;
        // Discard if not on screen
        if screen.top_left.x > bounds.bottom_right.x
            || screen.top_left.y > bounds.bottom_right.y
            || screen.bottom_right.x < bounds.top_left.x
            || screen.bottom_right.y < bounds.top_left.y
        {
            return;
        }

        // Room in screen space
        let screen_loc = Rectangle::new(
            bounds.top_left - screen.top_left,
            bounds.bottom_right - screen.top_left,
        );
        // top row (with culling)
        let top = screen_loc.top_left.y;
        if top >= 0 && top < screen.height() {
            let left = max(0, screen_loc.top_left.x);
            let right = min(screen.width(), screen_loc.bottom_right.x);
            for i in left..right {
                draw.set(top as usize, i as usize, char!('#'));
            }
        }
        // bottom row (with culling)
        let bottom = screen_loc.bottom_right.y - 1;
        if bottom >= 0 && bottom < screen.height() {
            let left = max(0, screen_loc.top_left.x);
            let right = min(screen.width(), screen_loc.bottom_right.x);
            for i in left..right {
                draw.set(bottom as usize, i as usize, char!('#'));
            }
        }
        // left column
        let left = screen_loc.top_left.x;
        if left >= 0 && left < screen.width() {
            let top = max(0, screen_loc.top_left.y);
            let bottom = min(screen.height(), screen_loc.bottom_right.y);
            for i in top..bottom {
                draw.set(i as usize, left as usize, char!('#'));
            }
        }
        // right column
        let right = screen_loc.bottom_right.x - 1;
        if right >= 0 && right < screen.width() {
            let top = max(0, screen_loc.top_left.y);
            let bottom = min(screen.height(), screen_loc.bottom_right.y);
            for i in top..bottom {
                draw.set(i as usize, right as usize, char!('#'));
            }
        }

        let left = max(0, screen_loc.top_left.x + 1);
        let right = min(screen.width(), screen_loc.bottom_right.x - 1);
        let top = max(0, screen_loc.top_left.y + 1);
        let bottom = min(screen.height(), screen_loc.bottom_right.y - 1);
        for x in left..right {
            for y in top..bottom {
                draw.set(y as usize, x as usize, char!('.'));
            }
        }
    }

    /// Get a random location in the room.
    pub fn random_location<R: Rng + ?Sized>(&self, rng: &mut R) -> Position<isize> {
        self.bounds.shrink(1).random_location(rng)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Player {
    location: Position<isize>
}

impl Player {
    pub fn new_unchecked(location: Position<isize>) -> Self {
        Player { location }
    }

    pub fn set_location_unchecked(&mut self, location: Position<isize>) {
        self.location = location
    }

    pub fn x(&self) -> isize {
        self.location.x
    }

    pub fn y(&self) -> isize {
        self.location.y
    }

    pub fn draw(&self, screen: Rectangle<isize>, draw: &mut Draw) {
        let screen_loc = self.location - screen.top_left;
        draw.set(screen_loc.y as usize, screen_loc.x as usize, char!('@'));
    }

    pub fn move_by(&mut self, direction: Direction, rooms: &[Room]) {
        let next_loc = match direction {
            Direction::Up => self.location.move_by((0, -1)),
            Direction::Down => self.location.move_by((0, 1)),
            Direction::Left => self.location.move_by((-1, 0)),
            Direction::Right => self.location.move_by((1, 0)),
        };
        if rooms.iter().any(|room| room.contains(next_loc)) {
            self.location = next_loc;
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Rectangle<T> {
    top_left: Position<T>,
    bottom_right: Position<T>,
}

impl<T> Rectangle<T>
where
    T: Ord
{
    pub fn new(top_left: Position<T>, bottom_right: Position<T>) -> Self {
        debug_assert!(top_left.x < bottom_right.x && top_left.y < bottom_right.y);
        Rectangle {
            top_left,
            bottom_right,
        }
    }
}

impl<T> Rectangle<T>
where
    T: Sub + Copy
{
    pub fn width(&self) -> <T as Sub>::Output {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> <T as Sub>::Output {
        self.bottom_right.y - self.top_left.y
    }
}

impl<T> Rectangle<T>
where
    T: Add<Output=T> + Sub<Output=T> + Ord + Copy
{
    pub fn shrink(self, amount: T) -> Self {
        let top_left = self.top_left + amount;
        let bottom_right = self.bottom_right - amount;
        if top_left.x >= bottom_right.x || top_left.y >= bottom_right.y {
            panic!("Rectangle too small to shrink");
        }
        Rectangle { top_left, bottom_right }
    }
}

impl<T> Rectangle<T>
where
    T: SampleUniform + SampleBorrow<T> + Copy
{
    /// Get a random location in the rectangle.
    pub fn random_location<R: Rng + ?Sized>(&self, rng: &mut R) -> Position<T> {
        Position {
            x: rng.gen_range(self.top_left.x, self.bottom_right.x),
            y: rng.gen_range(self.top_left.y, self.bottom_right.y),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Position<T> {
    x: T, // row
    y: T, // column
}

impl Position<usize> {
    fn to_signed(&self) -> Position<isize> {
        debug_assert!(self.x > isize::max_value() as usize || self.y > isize::max_value() as usize);
        Position {
            x: self.x as isize,
            y: self.y as isize,
        }
    }
}

impl<T> From<(T, T)> for Position<T> {
    fn from((x, y): (T, T)) -> Position<T> {
        Position { x, y }
    }
}

impl<T> Into<(T, T)> for Position<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> Position<T>
where
    T: Add
{
    pub fn move_by(self, (x, y): (T, T)) -> Position<<T as Add>::Output> {
        Position { x: self.x + x, y: self.y + y }
    }
}

impl<T> Add for Position<T>
where T: Add
{
    type Output = Position<<T as Add>::Output>;
    fn add(self, other: Self) -> Self::Output {
        Position {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl<T> Add<T> for Position<T>
where T: Add + Copy
{
    type Output = Position<<T as Add>::Output>;
    fn add(self, scalar: T) -> Self::Output {
        Position {
            x: self.x + scalar,
            y: self.y + scalar
        }
    }
}

impl<T> Sub for Position<T>
where T: Sub
{
    type Output = Position<<T as Sub>::Output>;
    fn sub(self, other: Self) -> Self::Output {
        Position {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl<T> Sub<T> for Position<T>
where T: Sub + Copy
{
    type Output = Position<<T as Sub>::Output>;
    fn sub(self, scalar: T) -> Self::Output {
        Position {
            x: self.x - scalar,
            y: self.y - scalar,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left
}
