use std::ops::{Add, Div, Mul, MulAssign, Sub};

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

    pub fn dot(self, other: Self) -> T
    where
        T: Add<Output = T> + Mul<Output = T>,
    {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Self) -> T
    where
        T: Sub<Output = T> + Mul<Output = T>,
    {
        self.x * other.y - self.y * other.x
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from(tuple: (T, T)) -> Self {
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
    pub fn shares_endpoint(&self, other: &Self) -> bool {
        (self.a == other.a) | (self.a == other.b) | (self.b == other.a) | (self.b == other.b)
    }
    pub fn extends(&self, other: &Self) -> bool {
        self.shares_endpoint(other) & (self.orientation() == other.orientation())
    }
}

impl<T> Line<T>
where
    T: num_traits::real::Real + std::cmp::PartialEq,
{
    fn ccw(a: &Point<T>, b: &Point<T>, c: &Point<T>) -> bool {
        (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
    }

    pub fn contains(&self, other: &Self) -> bool {
        (self.orientation() == other.orientation())
            & (self.a.x..=self.b.x).contains(&other.a.x)
            & (self.a.y..=self.b.y).contains(&other.a.y)
            & (self.a.x..=self.b.x).contains(&other.b.x)
            & (self.a.y..=self.b.y).contains(&other.b.y)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let p1 = &self.a;
        let p2 = &self.b;
        let p3 = &other.a;
        let p4 = &other.b;
        (Self::ccw(p1, p3, p4) != Self::ccw(p2, p3, p4))
            & (Self::ccw(p1, p2, p3) != Self::ccw(p1, p2, p4))
    }

    pub fn intersection(&self, other: &Self) -> Option<Point<T>> {
        let p1 = &self.a;
        let p2 = &self.b;
        let p3 = &other.a;
        let p4 = &other.b;
        let d = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
        let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / d;
        let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / d;
        if (T::from(0.0)..=T::from(1.0)).contains(&Some(t))
            && (T::from(0.0)..=T::from(1.0)).contains(&Some(u))
        {
            Some(Point::new(
                p1.x + t * (p2.x - p1.x),
                p1.y + t * (p2.y - p1.y),
            ))
        } else {
            None
        }
    }
}

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

impl<T> MulAssign<T> for Point<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
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

impl<T> MulAssign<T> for Line<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.a *= rhs;
        self.b *= rhs;
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
    pub fn snorm(&self) -> T {
        self.x.powi(2) + self.y.powi(2)
    }

    pub fn norm(&self) -> T {
        self.snorm().sqrt()
    }

    pub fn distance(&self, other: &Self) -> T {
        (*other - *self).norm()
    }
}
