use std::{io::Cursor, sync::{LazyLock, Arc}, ops::{Add, AddAssign, Index, IndexMut, Mul}};

use educe::Educe;
use num_enum::TryFromPrimitive;
use vello::{kurbo::Vec2, peniko::{Blob, ImageData, ImageAlphaType}};
use vello_common::pixmap::Pixmap;

pub const BOARD_SIZE:f64 = 800.;

mod position;

pub use position::Position;

// a recursive tic tac toe board
#[derive(Educe)]
#[educe(Default)]
pub struct BoardState {
    #[educe(Default=1)]
    pub layers: usize,
    pub board: LayerState,
}

#[derive(Clone, Copy, Default)]
pub enum Value {
    X, O, Tie, Some, #[default] Empty,
}

macro_rules! image {
    ($image:expr, $width:expr, $height:expr) => {
        {
            static DATA:LazyLock<Arc<Vec<u8>>> = LazyLock::new(|| {
                let data = Cursor::new(include_bytes!($image));
                Arc::new(Pixmap::from_png(data).unwrap().take_rgba8(ImageAlphaType::AlphaPremultiplied))
            });
            ImageData {
                data: Blob::new(DATA.clone()),
                format: vello::peniko::ImageFormat::Rgba8,
                alpha_type: vello::peniko::ImageAlphaType::Alpha,
                width: $width, height: $height,
            }
        }
    };
}

impl Value {
    pub fn image(self) -> Option<ImageData> {
        Some(match self {
            Value::X => image!("../../assets/x.png", 32, 32),
            Value::O => image!("../../assets/o.png", 32, 32),
            Value::Tie => image!("../../assets/tie.png", 32, 32),
            Value::Some => image!("../../assets/some.png", 32, 32),
            Value::Empty => return None
        })
    }
}

#[derive(Clone, Copy, TryFromPrimitive, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    TL, TM, TR,
    ML, MM, MR,
    BL, BM, BR,
}

impl Direction {
    pub fn next(self) -> Option<Direction> {
        (self as u8 + 1).try_into().ok()
    }
}

#[derive(Default)]
pub struct LayerState {
    pub value: Value,
    content: Option<LayerContent>
}

impl LayerState {
    pub fn expand_or_create(&mut self) {
        if let Some(content) = &mut self.content {
            content.expanded = true;
        } else {
            self.content = Some(LayerContent { expanded: true, further: Default::default() })
        }
    }
}

pub struct LayerContent {
    expanded: bool, // whether or not this is visibly expanded
    further: Box<[LayerState; 9]>,
}

impl BoardState {
    pub fn iter_terminal_visible<T:Clone, I, R>(&self, initialise:I, recurse:R) -> BoardIterator<'_, T, I, R>
    where I: FnOnce() -> T + Copy, R: Fn(&T, Direction) -> T {
        BoardIterator { board: &self.board, only_visible: true, curr: None, initialise, recurse, states: vec![] }
    }
}
pub struct BoardIterator<'a, T, I, R>
where I: FnOnce() -> T + Copy, R: Fn(&T, Direction) -> T {
    board: &'a LayerState,
    only_visible: bool,
    states: Vec<T>,
    initialise: I,
    recurse: R,
    curr: Option<Position>,
}

impl<T: Clone, I, R> Iterator for BoardIterator<'_, T, I, R>
where I: FnOnce() -> T + Copy, R: Fn(&T, Direction) -> T {
    type Item = (Value, T);

    fn next(&mut self) -> Option<Self::Item> {
        let (pos, value) = if let Some(pos) = self.curr.take() {
            // every iteration after the first

            // board is immutable while we have this so it shouldnt ever get out of sync
            let mut stack = pos.get_stack(self.board).unwrap();
            loop {
                let Some((direction, _)) = stack.pop() else { return None };
                self.states.pop();
                if let Some(next_direction) = direction.next() {
                    let previous_layer = if let Some((_, l)) = stack.last() { *l } else { self.board };
                    let next_layer = &previous_layer.content.as_ref().unwrap().further[next_direction];
                    stack.push((next_direction, next_layer));
                    self.states.push((self.recurse)(self.states.last().unwrap(), next_direction));
                    let mut layer_curr = next_layer;
                    loop {
                        match &layer_curr.content {
                            None => break,
                            Some(LayerContent {
                                expanded, further
                            }) => if self.only_visible && !expanded { break } else {
                                layer_curr = &further[Direction::TL];
                                stack.push((Direction::TL, layer_curr));
                                self.states.push((self.recurse)(self.states.last().unwrap(), Direction::TL));
                            }
                        }
                    }
                    break;
                }
            }
            let value = stack[stack.len()-1].1.value;
            (Position(stack.into_iter().map(|(d, _)| d).collect()), value)
        } else {
            // first iteration
            let mut pos_curr = Position::default();
            let mut layer_curr = self.board;
            self.states.push((self.initialise)());
            loop {
                match &layer_curr.content {
                    None => break (pos_curr, layer_curr.value),
                    Some(LayerContent {
                        expanded, further
                    }) => if self.only_visible && !expanded {
                        break (pos_curr, layer_curr.value)
                    } else {
                        layer_curr = &further[Direction::TL];
                        pos_curr += Direction::TL;
                        self.states.push((self.recurse)(self.states.last().unwrap(), Direction::TL));
                    }
                }
            }
        };
        self.curr = Some(pos);
        Some((value, self.states.last().unwrap().clone())) // is this clone necessary?
    }
}

impl Add<Direction> for Position {
    type Output = Position;
    fn add(self, rhs: Direction) -> Self::Output {
        Position(self.0.iter().copied().chain([rhs]).collect())
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        *self = self.clone() + rhs
    }
}

impl<T> Index<Direction> for [T; 9] {
    type Output = T;
    fn index(&self, index:Direction) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Direction> for [T; 9] {
    fn index_mut(&mut self, index:Direction) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl Mul<f64> for Direction {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Direction::TL => Vec2::new(-rhs, -rhs),
            Direction::TM => Vec2::new(0., -rhs),
            Direction::TR => Vec2::new(rhs, -rhs),
            Direction::ML => Vec2::new(-rhs, 0.),
            Direction::MM => Vec2::new(0., 0.),
            Direction::MR => Vec2::new(rhs, 0.),
            Direction::BL => Vec2::new(-rhs, rhs),
            Direction::BM => Vec2::new(0., rhs),
            Direction::BR => Vec2::new(rhs,rhs),
        }
    }
}

