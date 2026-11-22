use vello::kurbo::Vec2;

use crate::board::{Direction, LayerContent, LayerState};

// from big layers down to small layers
#[derive(Default, Clone, Debug)]
pub struct Position(pub(super) Box<[Direction]>);

#[derive(Debug, Clone, Copy)]
pub enum PositionError { PositionTooDeep, PositionTooShallow }

impl Position {
    /// returns a vec of directions, and then the resulting layer from that direction,
    /// so if its like [(d1, l1), (d2, l2)], then board[d1] = l1; l1[d2] = l2
    #[must_use]
    pub fn get_stack<'a>(&self, board:&'a LayerState) -> (Vec<(Direction, &'a LayerState)>, Option<PositionError>) {
        let pos = &self.0;
        let mut stack = vec![];
        let mut layer_curr = board;
        let mut pos_index = 0;
        loop {
            if let Some(LayerContent { further, .. }) = &layer_curr.content {
                let Some(direction) = pos.get(pos_index).copied() else { return (stack, Some(PositionError::PositionTooShallow)) };
                layer_curr = &further[direction];
                stack.push((direction, layer_curr));
                pos_index += 1;
            } else { break }
        }
        (stack, (pos_index != pos.len()).then_some(PositionError::PositionTooDeep))
    }

    #[must_use]
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
        let too_shallow = if let Some(LayerContent { expanded, .. }) = layer_curr.content { (!require_visible) || expanded } else { false };
        (layer_curr, too_shallow.then_some(PositionError::PositionTooShallow))
    }

    #[must_use]
    pub fn get_layer_mut<'a>(&self, board:&'a mut LayerState, require_visible:bool) -> (&'a mut LayerState, Option<PositionError>) {
        let pos = &self.0;
        let mut layer_curr = board;
        for direction in pos.iter().copied() {
            if let Some(LayerContent { ref mut further, .. }) = layer_curr.content {
                layer_curr = &mut further[direction];
            } else {
                return (layer_curr, Some(PositionError::PositionTooDeep));
            }
        }
        let too_shallow = if let Some(LayerContent { expanded, .. }) = layer_curr.content { (!require_visible) || expanded } else { false };
        (layer_curr, too_shallow.then_some(PositionError::PositionTooShallow))
    }

    #[must_use]
    pub fn matches_at(&self, check:Direction, index:usize) -> bool {
        self.0.get(index).is_some_and(|d| *d == check)
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.0.len()
    }

    /// scaled such that it is from 0 to 1
    #[must_use]
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

    #[must_use]
    pub fn out_one(&self) -> Option<Position> {
        if self.0.len() == 0 { return None }
        Some(Position(self.0[..self.0.len() - 1].iter().copied().collect()))
    }

    #[must_use]
    pub fn without(&self, index:usize) -> Option<Position> {
        if self.0.len() == 0 { return None }
        Some(Position(self.0[..index].iter().chain(&self.0[index+1..]).copied().collect()))
    }

    #[must_use]
    pub fn within(&self, other:&Position) -> bool {
        if self.0.len() < other.0.len() { return false }
        self.0.iter().zip(other.0.iter()).all(|(this, that)| *this == *that)
    }
}

impl From<Vec<Direction>> for Position {
    fn from(value: Vec<Direction>) -> Self {
        Position(value.into())
    }
}

