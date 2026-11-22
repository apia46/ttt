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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
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
    #[must_use]
    pub fn image(self) -> Option<ImageData> {
        Some(match self {
            Value::X => image!("../../assets/x.png", 32, 32),
            Value::O => image!("../../assets/o.png", 32, 32),
            Value::Tie => image!("../../assets/tie.png", 32, 32),
            Value::Some => image!("../../assets/some.png", 32, 32),
            Value::Empty => return None
        })
    }

    #[must_use]
    pub fn is_full(self) -> bool {
        match self {
            Value::X | Value::O | Value::Tie => true,
            _ => false,
        }
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
    #[must_use]
    pub fn next(self) -> Option<Direction> {
        (self as u8 + 1).try_into().ok()
    }
}

#[derive(Default)]
pub struct LayerState {
    pub value: Value,
    pub content: Option<LayerContent>
}

const LINES:[[Direction; 3]; 8] = [
    [Direction::TL, Direction::TM, Direction::TR],
    [Direction::ML, Direction::MM, Direction::MR],
    [Direction::BL, Direction::BM, Direction::MR],
    [Direction::TL, Direction::ML, Direction::BL],
    [Direction::TM, Direction::MM, Direction::BM],
    [Direction::TR, Direction::MR, Direction::BR],
    [Direction::TL, Direction::MM, Direction::BR],
    [Direction::TR, Direction::MM, Direction::BL],
];

impl LayerState {
    pub fn expand_or_create(&mut self) -> &mut LayerContent {
        if let Some(ref mut content) = self.content {
            content.expanded = true;
            content
        } else {
            self.content = Some(LayerContent { expanded: true, further: Default::default() });
            self.content.as_mut().unwrap()
        }
    }

    pub fn expand_or_create_to(&mut self, pos:&Position) -> &mut LayerState {
        let mut layer_curr = self;
        for direction in &pos.0 {
            if let Some(ref mut content) = layer_curr.content {
                content.expanded = true;
                layer_curr = &mut content.further[*direction];
            } else {
                layer_curr.content = Some(LayerContent { expanded: true, further: Default::default() });
                layer_curr = &mut layer_curr.content.as_mut().unwrap().further[*direction];
            }
        }
        layer_curr
    }

    /// returns the new value if it was changed
    pub fn update_value_from_content(&mut self) -> Option<Value> {
        let prev_value = self.value;
        let Some(next_value) = self.value_from_content() else { return None };
        self.value = next_value;
        (prev_value == next_value).then_some(next_value)
    }

    #[must_use]
    pub fn value_from_content(&self) -> Option<Value> {
        let Some(LayerContent { further, .. }) = &self.content else { return None };
        for line in LINES {
            let values = line.iter().map(|d| further[*d].value);
            if values.clone().all(|v| v == Value::X) { return Some(Value::X) }
            if values.clone().all(|v| v == Value::O) { return Some(Value::O) }
        }
        if further.iter().all(|l| l.value == Value::Empty ) { Some(Value::Empty) }
        else { Some(Value::Some) }
    }
}

pub struct LayerContent {
    expanded: bool, // whether or not this is visibly expanded
    further: Box<[LayerState; 9]>,
}

impl BoardState {
    pub fn expand(&mut self) {
        self.layers += 1;
        let previous = std::mem::take(&mut self.board);
        self.board.expand_or_create().further[Direction::MM] = previous;
    }

    #[must_use]
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

