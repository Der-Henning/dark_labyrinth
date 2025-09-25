use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::geometrie::{Line, Point};
use crate::{RAY_LENGTH, RAYS, WINDOW_DIMENSIONS};

pub struct Game {
    pub position: Point<f32>,
    pub target: Point<f32>,
    pub timer: GameTimer,
    pub walls: Vec<Line<f32>>,
    pub grid_size: usize,
    grid: Grid,
    base_rays: Vec<Point<f32>>,
    threshold: f32,
}

impl Game {
    pub fn new(grid_size: usize, dropout: f32, target_threshold: usize) -> Self {
        let walls = make_walls(grid_size, dropout);
        let grid = Grid::new(grid_size).fill(&walls);

        Self {
            position: get_random_point(grid_size),
            target: get_random_point(grid_size),
            timer: GameTimer::new(),
            walls,
            grid_size,
            grid,
            base_rays: get_ray_directions(RAYS, (grid_size * RAY_LENGTH) as f32),
            threshold: (grid_size / target_threshold) as f32,
        }
    }

    pub fn update_position(&mut self) {
        let mouse_position = Point::from(mouse_position());
        let new_position = self.position + (mouse_position - self.position) * 0.1;
        let direction = Line::new(self.position, new_position);
        let cell = self.grid.find(&self.position);

        match self
            .grid
            .find_intersection(&direction, cell, Direction::None)
        {
            Some(p) => {
                let direction = p - self.position;
                let distance = direction.norm();
                self.position = self.position + direction * (distance - 1.0) / distance;
            }
            _ => self.position = new_position,
        }
    }

    pub fn get_rays(&self) -> Vec<Point<f32>> {
        let cell = self.grid.find(&self.position);

        self.base_rays
            .iter()
            .map(|&r| {
                let p2 = self.position + r;
                let ray = Line::new(self.position, p2);
                match self.grid.find_intersection(&ray, cell, Direction::None) {
                    Some(p) => p,
                    _ => p2,
                }
            })
            .collect()
    }

    pub fn found_target(&self) -> bool {
        self.position.distance(&self.target) < self.threshold
    }
}

fn get_random_point(grid_size: usize) -> Point<f32> {
    Point::new(
        rand::rand() as usize % (WINDOW_DIMENSIONS.x as usize / grid_size) * grid_size
            + grid_size / 2,
        rand::rand() as usize % (WINDOW_DIMENSIONS.y as usize / grid_size) * grid_size
            + grid_size / 2,
    )
    .into()
}

fn make_walls(grid_size: usize, dropout: f32) -> Vec<Line<f32>> {
    let labyrinth = compress_labyrinth(make_labyrinth(grid_size, dropout));
    labyrinth
        .into_iter()
        .map(|line| Line::<f32>::from(line * grid_size))
        .collect()
}

fn make_labyrinth(grid_size: usize, dropout: f32) -> Vec<Line<usize>> {
    type Area = HashSet<Point<usize>>;
    type Edge = (usize, Option<usize>);

    let mut areas: HashMap<usize, Area> = HashMap::new();
    let mut edges: HashMap<Line<usize>, Edge> = HashMap::new();

    (0..WINDOW_DIMENSIONS.x as usize / grid_size)
        .cartesian_product(0..WINDOW_DIMENSIONS.y as usize / grid_size)
        .map(|(x, y)| Point::new(x, y))
        .enumerate()
        .for_each(|(area_id, cell)| {
            areas.insert(area_id, HashSet::from([cell]));

            let new_edges = vec![
                Line::new(cell, Point::new(cell.x + 1, cell.y)),
                Line::new(cell, Point::new(cell.x, cell.y + 1)),
                Line::new(
                    Point::new(cell.x + 1, cell.y),
                    Point::new(cell.x + 1, cell.y + 1),
                ),
                Line::new(
                    Point::new(cell.x, cell.y + 1),
                    Point::new(cell.x + 1, cell.y + 1),
                ),
            ];

            for edge in new_edges {
                edges
                    .entry(edge)
                    .and_modify(|e| e.1 = Some(area_id))
                    .or_insert((area_id, None));
            }
        });

    let mut inner_edges = edges
        .iter()
        .filter_map(|(k, v)| v.1.map(|_| *k))
        .collect::<Vec<_>>();

    while areas.len() > 1 {
        inner_edges.retain(|edge_id| {
            let edge = edges.get(edge_id).unwrap();
            edge.0 != edge.1.unwrap()
        });

        let rng_edge_idx = rand::rand() as usize % inner_edges.len();
        let edge_id = inner_edges.swap_remove(rng_edge_idx);
        let edge = edges.remove(&edge_id).unwrap();
        let right_area = areas.remove(&edge.1.unwrap()).unwrap();

        areas
            .entry(edge.0)
            .and_modify(|a| a.extend(right_area.into_iter()));

        edges.iter_mut().for_each(|(_k, v)| {
            if v.0 == edge.1.unwrap() {
                v.0 = edge.0
            }
            if v.1.is_some() && v.1.unwrap() == edge.1.unwrap() {
                v.1 = Some(edge.0)
            }
        });
    }

    inner_edges = edges
        .iter()
        .filter_map(|(k, v)| v.1.map(|_v| *k))
        .collect::<Vec<_>>();

    (0..(inner_edges.len() as f32 * dropout) as usize).for_each(|_| {
        let rng_edge_idx = rand::rand() as usize % inner_edges.len();
        let edge_id = inner_edges.swap_remove(rng_edge_idx);
        edges.remove(&edge_id);
    });

    edges.into_keys().collect()
}

fn compress_labyrinth(mut labyrinth: Vec<Line<usize>>) -> Vec<Line<usize>> {
    let mut zipped_labyrinth: Vec<Line<usize>> = Vec::new();

    while let Some(wall) = labyrinth.pop() {
        match labyrinth.iter().position(|w| w.extends(&wall)) {
            Some(idx) => {
                let wall2 = labyrinth.swap_remove(idx);
                let points = [wall.a, wall.b, wall2.a, wall2.b];
                labyrinth.push(Line::new(
                    *points.iter().min().unwrap(),
                    *points.iter().max().unwrap(),
                ));
            }
            _ => zipped_labyrinth.push(wall),
        }
    }

    zipped_labyrinth
}

fn get_ray_directions(rays: usize, length: f32) -> Vec<Point<f32>> {
    (0..360)
        .step_by(360 / rays)
        .map(|r| r as f32 / 360.0 * 2.0 * std::f32::consts::PI)
        .map(|r| Point::new(r.sin(), r.cos()) * length)
        .collect()
}

#[derive(PartialEq, Eq, Hash, Debug)]
enum Direction {
    None,
    North,
    East,
    South,
    West,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

impl Direction {
    fn rev(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::None => Self::None,
        }
    }
}

#[derive(Debug)]
struct Cell {
    position: Point<usize>,
    borders: HashMap<Direction, Line<f32>>,
    walls: HashMap<Direction, Line<f32>>,
}

impl Cell {
    fn new(x: usize, y: usize, grid_size: usize) -> Self {
        let p1 = Point::new(x * grid_size, y * grid_size).into();
        let p2 = Point::new((x + 1) * grid_size, y * grid_size).into();
        let p3 = Point::new((x + 1) * grid_size, (y + 1) * grid_size).into();
        let p4 = Point::new(x * grid_size, (y + 1) * grid_size).into();
        Self {
            position: Point::new(x, y),
            borders: HashMap::from([
                (Direction::North, Line::new(p1, p2)),
                (Direction::East, Line::new(p2, p3)),
                (Direction::South, Line::new(p3, p4)),
                (Direction::West, Line::new(p1, p4)),
            ]),
            walls: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct Grid {
    cells: HashMap<(usize, usize), Cell>,
    grid_size: usize,
}

impl Grid {
    fn new(grid_size: usize) -> Self {
        Self {
            cells: (0..WINDOW_DIMENSIONS.x as usize / grid_size)
                .cartesian_product(0..WINDOW_DIMENSIONS.y as usize / grid_size)
                .map(|(x, y)| ((x, y), Cell::new(x, y, grid_size)))
                .collect(),
            grid_size,
        }
    }

    fn fill(mut self, walls: &[Line<f32>]) -> Self {
        for (_, c) in self.cells.iter_mut() {
            for wall in walls {
                for direction in DIRECTIONS {
                    if wall.contains(&c.borders[&direction]) {
                        c.walls.insert(direction, *wall);
                        break;
                    }
                }
            }
        }
        self
    }

    fn move_to<'a>(&'a self, cell: &'a Cell, direction: &Direction) -> &'a Cell {
        match direction {
            Direction::North => &self.cells[&(cell.position.x, cell.position.y - 1)],
            Direction::East => &self.cells[&(cell.position.x + 1, cell.position.y)],
            Direction::South => &self.cells[&(cell.position.x, cell.position.y + 1)],
            Direction::West => &self.cells[&(cell.position.x - 1, cell.position.y)],
            Direction::None => cell,
        }
    }

    fn find(&self, p: &Point<f32>) -> &Cell {
        &self.cells[&(p.x as usize / self.grid_size, p.y as usize / self.grid_size)]
    }

    fn find_intersection(
        &self,
        line: &Line<f32>,
        cell: &Cell,
        direction: Direction,
    ) -> Option<Point<f32>> {
        for dir in DIRECTIONS {
            if dir != direction.rev() && line.intersects(&cell.borders[&dir]) {
                if let Some(w) = cell.walls.get(&dir) {
                    return line.intersection(w);
                } else {
                    let next_cell = self.move_to(cell, &dir);
                    return self.find_intersection(line, next_cell, dir);
                }
            }
        }
        None
    }
}

enum GameTimerState {
    Idle,
    Running,
    Paused,
}

pub struct GameTimer {
    times: Vec<f64>,
    instant: Option<f64>,
    state: GameTimerState,
    pub result: Option<f64>,
}

impl GameTimer {
    pub fn new() -> Self {
        Self {
            times: Vec::new(),
            instant: None,
            state: GameTimerState::Idle,
            result: None,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            GameTimerState::Idle => {
                self.times = Vec::new();
                self.instant = Some(macroquad::miniquad::date::now());
                self.state = GameTimerState::Running;
            }
            _ => panic!("Can only start game timer in idle mode!"),
        }
    }

    pub fn current(&self) -> f64 {
        match self.state {
            GameTimerState::Running => match self.instant {
                Some(i) => self.times.iter().sum::<f64>() + macroquad::miniquad::date::now() - i,
                _ => self.times.iter().sum(),
            },
            _ => self.times.iter().sum(),
        }
    }

    pub fn stop(&mut self) {
        match self.state {
            GameTimerState::Running => {
                self.times
                    .push(macroquad::miniquad::date::now() - self.instant.unwrap());
                self.result = Some(self.times.iter().sum());
                self.state = GameTimerState::Idle;
            }
            GameTimerState::Paused => {
                self.result = Some(self.times.iter().sum());
                self.state = GameTimerState::Idle;
            }
            GameTimerState::Idle => panic!("Cannot stop idle game timer!"),
        }
    }

    pub fn pause(&mut self) {
        match self.state {
            GameTimerState::Running => {
                self.times
                    .push(macroquad::miniquad::date::now() - self.instant.unwrap());
                self.state = GameTimerState::Paused;
            }
            _ => panic!("Can only pause game timer in running state!"),
        }
    }

    pub fn resume(&mut self) {
        match self.state {
            GameTimerState::Paused => {
                self.instant = Some(macroquad::miniquad::date::now());
                self.state = GameTimerState::Running;
            }
            _ => panic!("Can only resume game timer in paused mode!"),
        }
    }
}
