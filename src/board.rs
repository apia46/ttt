use std::ops::{Add, AddAssign, Index};

use educe::Educe;
use num_enum::TryFromPrimitive;

#[derive(Clone, Copy)]
pub enum Value {
    X, O, Tie, Some, Empty,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Direction {
    TL, T, TR,
    L,  M, R,
    BL, B, BR,
}

impl Direction {
    pub fn next(self) -> Option<Direction> {
        (self as u8 + 1).try_into().ok()
    }
}

// from big layers down to small layers
#[derive(Default, Clone)]
pub struct Position(Box<[Direction]>);

#[derive(Educe)]
#[educe(Default)]
pub struct BoardState {
    #[educe(Default=1)]
    pub layers: usize,
    pub board: LayerState,
}

pub struct LayerState {
    value: Value,
    content: Option<LayerContent>
}

pub struct LayerContent {
    expanded: bool, // whether or not this is visibly expanded
    further: Box<[LayerState; 9]>,
}


impl BoardState {
    pub fn iter_terminal_visible(&self) -> BoardIterator<'_> {
        BoardIterator { board: &self.board, only_visible: true, curr: None }
    }
}

impl Default for LayerState {
    fn default() -> Self {
        LayerState {
            value: Value::Empty,
            content: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PositionError { PositionTooDeep, PositionTooShallow }

impl Position {
    /// returns a vec of directions, and then the resulting layer from that direction,
    /// so if its like [(d1, l1), (d2, l2)], then board[d1] = l1; l1[d2] = l2
    pub fn get_stack(self, board:&LayerState) -> Result<Vec<(Direction, &LayerState)>, PositionError> {
        let pos = self.0;
        let mut stack = vec![];
        let mut layer_curr = board;
        let mut pos_index = 0;
        loop {
            if let Some(LayerContent { further, .. }) = &layer_curr.content {
                if pos_index >= pos.len() { return Err(PositionError::PositionTooShallow) }
                let direction = pos[pos_index];
                layer_curr = &further[direction];
                stack.push((direction, layer_curr));
                pos_index += 1;
            } else { break }
        }
        if pos_index != pos.len() { return Err(PositionError::PositionTooDeep) }
        Ok(stack)
    }
}

pub struct BoardIterator<'a> {
    board: &'a LayerState,
    only_visible: bool,
    curr: Option<Position>,
}

impl Iterator for BoardIterator<'_> {
    type Item = (Value, Position);

    fn next(&mut self) -> Option<Self::Item> {
        let (pos, value) = if let Some(pos) = self.curr.take() {
            // board is immutable while we have this so it shouldnt ever get out of sync
            let mut stack = pos.get_stack(self.board).unwrap();
            loop {
                let Some((direction, _)) = stack.pop() else { return None };
                if let Some(next_direction) = direction.next() {
                    let previous_layer = match stack.last() {
                        Some((_, l)) => *l,
                        None => self.board,
                    };
                    let next_layer = &previous_layer.content.as_ref().unwrap().further[next_direction];
                    stack.push((next_direction, next_layer));
                    let mut layer_curr = next_layer;
                    loop {
                        match &layer_curr.content {
                            None => break,
                            Some(LayerContent {
                                expanded, further
                            }) => if self.only_visible && !expanded { break } else {
                                layer_curr = &further[Direction::TL];
                                stack.push((Direction::TL, layer_curr));
                            }
                        }
                    }
                    break;
                }
            }
            let value = stack[stack.len()-1].1.value;
            (Position(stack.into_iter().map(|(d, _)| d).collect()), value)
        } else {
            let mut pos_curr = Position::default();
            let mut layer_curr = self.board;
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
                    }
                }
            }
        };
        self.curr = Some(pos.clone());
        Some((value, pos))
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

