use std::ops::{Add, AddAssign, Index, IndexMut, Mul};

use educe::Educe;
use num_enum::TryFromPrimitive;
use vello::kurbo::Vec2;

pub const BOARD_SIZE:f64 = 800.;

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

// from big layers down to small layers
#[derive(Default, Clone, Debug)]
pub struct Position(Box<[Direction]>);

#[derive(Default)]
pub struct LayerState {
    value: Value,
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

#[derive(Debug, Clone, Copy)]
pub enum PositionError { PositionTooDeep, PositionTooShallow }

impl Position {
    /// returns a vec of directions, and then the resulting layer from that direction,
    /// so if its like [(d1, l1), (d2, l2)], then board[d1] = l1; l1[d2] = l2
    pub fn get_stack<'a>(&self, board:&'a LayerState) -> Result<Vec<(Direction, &'a LayerState)>, PositionError> {
        let pos = &self.0;
        let mut stack = vec![];
        let mut layer_curr = board;
        let mut pos_index = 0;
        loop {
            if let Some(LayerContent { further, .. }) = &layer_curr.content {
                let Some(direction) = pos.get(pos_index).copied() else { return Err(PositionError::PositionTooShallow) };
                layer_curr = &further[direction];
                stack.push((direction, layer_curr));
                pos_index += 1;
            } else { break }
        }
        if pos_index != pos.len() { return Err(PositionError::PositionTooDeep) }
        Ok(stack)
    }

    pub fn get_layer<'a>(&self, board:&'a LayerState, require_visible:bool) -> (&'a LayerState, Option<PositionError>) {
        let pos = &self.0;
        let mut layer_curr = board;
        for direction in pos.iter().copied() {
            if let Some(LayerContent { further, .. }) = &layer_curr.content {
                layer_curr = &further[direction];
            } else {
                return (layer_curr, Some(PositionError::PositionTooDeep));
            }
        }
        let too_shallow = if let Some(LayerContent { expanded, .. }) = layer_curr.content { expanded || !require_visible } else { false };
        (layer_curr, too_shallow.then_some(PositionError::PositionTooShallow))
    }

    pub fn get_layer_mut<'a>(&self, board:&'a mut LayerState, require_visible:bool) -> (&'a mut LayerState, Option<PositionError>) {
        let pos = &self.0;
        let mut layer_curr = board;
        for direction in pos.iter().copied() {
            if let Some(LayerContent { further, .. }) = &mut layer_curr.content {
                layer_curr = &mut further[direction];
            } else {
                panic!()
                // waiting for polonius
                //return (layer_curr, Some(PositionError::PositionTooDeep));
            }
        }
        let too_shallow = if let Some(LayerContent { expanded, .. }) = layer_curr.content { expanded || !require_visible } else { false };
        (layer_curr, too_shallow.then_some(PositionError::PositionTooShallow))
    }

    pub fn matches_at(&self, check:Direction, index:usize) -> bool {
        self.0.get(index).is_some_and(|d| *d == check)
    }

    /// scaled such that it is from 0 to 1
    pub fn from_vec2_visible(mut vec2: Vec2, board:&LayerState) -> Option<Position> {
        if vec2.x < 0. || vec2.x >= 1. || vec2.y < 0. || vec2.y >= 1. {
            return None;
        }

        const F13:f64 = 1./3.;
        const F23:f64 = 2./3.;

        let mut pos:Vec<Direction> = vec![];
        let mut layer = board;
        loop {
            let Some(LayerContent { expanded: true, further }) = &layer.content else { break };
            let dir = match (vec2.x, vec2.y) {
                (..F13, ..F13) => Direction::TL,
                (F13..F23, ..F13) => Direction::TM,
                (F23.., ..F13) => Direction::TR,
                (..F13, F13..F23) => Direction::ML,
                (F13..F23, F13..F23) => Direction::MM,
                (F23.., F13..F23) => Direction::MR,
                (..F13, F23..) => Direction::BL,
                (F13..F23, F23..) => Direction::BM,
                (F23.., F23..) => Direction::BR,
                _ => return None,
            };
            pos.push(dir);
            vec2 *= 3.;
            vec2 -= dir * 1. + Vec2::new(1., 1.);
            layer = &further[dir];
        }
        Some(Position(pos.into()))
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

