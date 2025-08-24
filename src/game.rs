use itertools::Itertools;
use macroquad::prelude::*;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::{
    DROPOUT, GRID_SIZES, Line, Point, RAY_LENGTH, RAYS, TARGET_THRESHOLD, WINDOW_DIMENSIONS,
};

#[derive(Clone)]
pub struct GameSettings {
    pub draw_labyrinth: bool,
    pub draw_fps: bool,
    pub labyrinth_size: usize,
}

impl GameSettings {
    pub fn new(draw_labyrinth: bool, draw_fps: bool, labyrinth_size: usize) -> Self {
        Self {
            draw_labyrinth,
            draw_fps,
            labyrinth_size,
        }
    }
}

pub struct Game {
    pub position: Point<f32>,
    pub target: Point<f32>,
    pub timer: GameTimer,
    pub settings: GameSettings,
    pub walls: Vec<Line<f32>>,
    pub grid_size: usize,
    base_rays: Vec<Point<f32>>,
}

impl Game {
    pub fn new(settings: GameSettings) -> Self {
        let grid_size = GRID_SIZES[settings.labyrinth_size];
        Self {
            position: get_random_point(grid_size),
            target: get_random_point(grid_size),
            timer: GameTimer::new(),
            settings,
            walls: make_walls(grid_size),
            grid_size,
            base_rays: get_ray_directions(),
        }
    }

    pub fn update_position(&mut self) {
        let mouse_position = Point::from(mouse_position());
        let new_position = self.position + (mouse_position - self.position) * 0.1;

        match get_next_wall_intersection(&self.position, &new_position, &self.walls) {
            Some((_w, p)) => {
                let direction = p - self.position;
                let distance = direction.norm();
                self.position = self.position + direction * (distance - 1.0) / distance;
            }
            _ => self.position = new_position,
        }
    }

    pub fn get_rays(&self) -> Vec<Point<f32>> {
        self.base_rays
            .par_iter()
            .map(|&r| {
                let p2 = self.position + r * (self.grid_size * RAY_LENGTH) as f32;
                match get_next_wall_intersection(&self.position, &p2, &self.walls) {
                    Some((_wall, p)) => p,
                    _ => p2,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn found_target(&self) -> bool {
        self.position.distance(&self.target)
            < (GRID_SIZES[self.settings.labyrinth_size] / TARGET_THRESHOLD) as f32
    }
}

fn get_random_point(grid_size: usize) -> Point<f32> {
    Point::new(
        ((rand::rand() as usize % (WINDOW_DIMENSIONS.1 / grid_size) * grid_size) + grid_size / 2)
            as f32,
        ((rand::rand() as usize % (WINDOW_DIMENSIONS.0 / grid_size) * grid_size) + grid_size / 2)
            as f32,
    )
}

fn make_walls(grid_size: usize) -> Vec<Line<f32>> {
    let labyrinth = compress_labyrinth(make_labyrinth(grid_size));
    labyrinth
        .into_iter()
        .map(|line| Line::<f32>::from(line * grid_size))
        .collect::<Vec<_>>()
}

pub fn get_next_wall_intersection(
    p1: &Point<f32>,
    p2: &Point<f32>,
    walls: &[Line<f32>],
) -> Option<(Line<f32>, Point<f32>)> {
    walls
        .iter()
        // filter for walls that have an intersection with the given ray to reduce computation
        .filter(|wall| {
            let p3 = wall.a;
            let p4 = wall.b;
            let ccw = |a: &Point<f32>, b: &Point<f32>, c: &Point<f32>| {
                (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
            };
            (ccw(p1, &p3, &p4) != ccw(p2, &p3, &p4)) & (ccw(p1, p2, &p3) != ccw(p1, p2, &p4))
        })
        // calculate the intersection point
        .map(|wall| {
            let p3 = wall.a;
            let p4 = wall.b;
            let d = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
            let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / d;
            let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / d;
            (wall, t, u)
        })
        .filter(|(_wall, t, u)| (&0.0..=&1.0).contains(&t) & (&0.0..=&1.0).contains(&u))
        .map(|(wall, t, _u)| (wall, p1.x + t * (p2.x - p1.x), p1.y + t * (p2.y - p1.y)))
        // take only the nearest intersection point to the player and return intersection as Point
        .map(|(wall, x, y)| (wall, x, y, (x - p1.x).powi(2) + (y - p1.y).powi(2)))
        .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
        .map(|(&wall, x, y, _d)| (wall, Point::new(x, y)))
}

fn make_labyrinth(grid_size: usize) -> Vec<Line<usize>> {
    type Area = HashSet<Point<usize>>;
    type Edge = (usize, Option<usize>);

    let mut areas: HashMap<usize, Area> = HashMap::new();
    let mut edges: HashMap<Line<usize>, Edge> = HashMap::new();

    (0..WINDOW_DIMENSIONS.1 / grid_size)
        .cartesian_product(0..WINDOW_DIMENSIONS.0 / grid_size)
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

    (0..(inner_edges.len() as f32 * DROPOUT) as usize).for_each(|_| {
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

fn get_ray_directions() -> Vec<Point<f32>> {
    (0..360)
        .step_by(360 / RAYS)
        .map(|r| r as f32 / 360.0 * 2.0 * std::f32::consts::PI)
        .map(|r| Point::new(r.sin(), r.cos()))
        .collect::<Vec<_>>()
}

enum GameTimerState {
    Idle,
    Running,
    Paused,
}

pub struct GameTimer {
    times: Vec<Duration>,
    instant: Option<Instant>,
    state: GameTimerState,
    pub result: Option<Duration>,
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
                self.instant = Some(Instant::now());
                self.state = GameTimerState::Running;
            }
            _ => panic!("Can only start game timer in idle mode!"),
        }
    }

    pub fn current(&self) -> Duration {
        match self.state {
            GameTimerState::Running => match self.instant {
                Some(i) => self.times.iter().sum::<Duration>() + i.elapsed(),
                _ => self.times.iter().sum::<Duration>(),
            },
            _ => self.times.iter().sum::<Duration>(),
        }
    }

    pub fn stop(&mut self) {
        match self.state {
            GameTimerState::Running => {
                self.times.push(self.instant.unwrap().elapsed());
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
                self.times.push(self.instant.unwrap().elapsed());
                self.state = GameTimerState::Paused;
            }
            _ => panic!("Can only pause game timer in running state!"),
        }
    }

    pub fn resume(&mut self) {
        match self.state {
            GameTimerState::Paused => {
                self.instant = Some(Instant::now());
                self.state = GameTimerState::Running;
            }
            _ => panic!("Can only resume game timer in paused mode!"),
        }
    }
}
