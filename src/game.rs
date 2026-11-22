use crate::board::{BoardState, LayerState, Position, Value};

#[derive(Default, Debug, Clone, Copy)]
pub enum Player { #[default] X, O }
impl Player {
    pub fn to_value(self) -> Value {
        match self {
            Player::X => Value::X,
            Player::O => Value::O,
        }
    }
}

#[derive(Default)]
pub struct GameState {
    pub other: GameOtherState,
    pub board: BoardState,
}

// hello code reviewer :)
#[derive(Default)]
pub struct GameOtherState {
    pub turn: Player,
}

impl GameOtherState {
    pub fn place_at(&mut self, layer:&mut LayerState) {
        layer.value = self.turn.to_value();
        self.turn = match self.turn {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

