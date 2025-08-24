use std::ops::{Add, Div, Mul, Sub};

#[derive(Hash, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

#[derive(Hash, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub struct Line<T> {
    pub a: Point<T>,
    pub b: Point<T>,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
    pub fn from(tuple: (T, T)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<Point<usize>> for Point<f32> {
    fn from(p: Point<usize>) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

impl From<Line<usize>> for Line<f32> {
    fn from(l: Line<usize>) -> Self {
        Self {
            a: l.a.into(),
            b: l.b.into(),
        }
    }
}

#[derive(PartialEq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

impl<T: std::cmp::PartialEq> Line<T> {
    pub fn new(a: Point<T>, b: Point<T>) -> Self {
        Self { a, b }
    }
    pub fn orientation(&self) -> Orientation {
        match self.a.x == self.b.x {
            true => Orientation::Horizontal,
            false => Orientation::Vertical,
        }
    }
    pub fn intersect(&self, other: &Self) -> bool {
        (self.a == other.a) | (self.a == other.b) | (self.b == other.a) | (self.b == other.b)
    }
    pub fn extends(&self, other: &Self) -> bool {
        self.intersect(other) & (self.orientation() == other.orientation())
    }
}

// Implement Add for Point<T> where T supports Add
impl<T> Add for Point<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Implement Sub for Point<T> where T supports Sub
impl<T> Sub for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Implement Point<T> * T
impl<T> Mul<T> for Point<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Div<T> for Point<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> Mul<T> for Line<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Line {
            a: self.a * rhs,
            b: self.b * rhs,
        }
    }
}

impl Eq for Point<usize> {}
impl Eq for Line<usize> {}

impl Ord for Point<usize> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.x, self.y).cmp(&(other.x, other.y))
    }
}

impl Ord for Line<usize> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.a, self.b).cmp(&(other.a, other.b))
    }
}

impl<T> Point<T>
where
    T: num_traits::real::Real,
{
    pub fn norm(&self) -> T {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    pub fn distance(&self, other: &Self) -> T {
        (*other - *self).norm()
    }
}
