use crate::board::{BoardState, LayerState, Position, PositionError, Value};

#[derive(Default, Debug, Clone, Copy)]
pub enum Player { #[default] X, O }
impl Player {
    #[must_use]
    pub fn to_value(self) -> Value {
        match self {
            Player::X => Value::X,
            Player::O => Value::O,
        }
    }
}

impl From<Value> for Option<Option<Player>> {
    fn from(value: Value) -> Self {
        match value {
            Value::X => Some(Some(Player::X)),
            Value::O => Some(Some(Player::O)),
            Value::Tie => Some(None),
            _ => None,
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
    pub domain: Position,
    pub state: GameStateState,
}

#[derive(Default)]
pub enum GameStateState {
    #[default] Play,
    ExpandPrompt(Position),
    End(Option<Player>),
}

impl GameOtherState {
    pub fn place_at(&mut self, layer:&mut LayerState) {
        layer.value = self.turn.to_value();
    }
    
    fn next_player(&mut self) {
        self.turn = match self.turn {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl GameState {
    pub fn placed_at(&mut self, position:&Position) {
        let mut pos_curr = position.out_one().unwrap();
        let mut layer_curr:&mut LayerState;
        let mut max_layer_won = 0;
        let mut max_layer_filled = 0;
        loop {
            layer_curr = if let (l, None | Some(PositionError::PositionTooShallow)) = pos_curr.get_layer_mut(&mut self.board.board, false) { l } else { panic!() };
            if let Some(value) = layer_curr.update_value_from_content() {
                if value == self.other.turn.to_value() { max_layer_won += 1; }
                if value.is_full() {
                    max_layer_filled += 1;
                    layer_curr.content.as_mut().unwrap().expanded = false;
                }
            }
            pos_curr = if let Some(p) = pos_curr.out_one() { p } else { break };
        }
        let this_player = self.other.turn;
        self.other.next_player();
        // if for example a [TL,TM,TR,ML] (in a layer 4 game) wins
        // the t0: you go to the t2 [TL,TM], append the winning [ML]
        // the t1: you go to the t3 [TL], append the winning [TR,ML]
        // the t2: you go to the t4 [], append the winning [TM,TR,TL]
        // the t3: if expand: now you go to [TL,TM,TR,ML] in the t5
        // if you dont expand, then you still go to [TM,TR,TL]
        // the t4: the game ends
        // if you filled more layers than you won...
        // what should happen?
        self.other.state = if max_layer_filled > max_layer_won && max_layer_filled == self.board.layers {
            // tie
            GameStateState::End(None)
        } else if max_layer_won == self.board.layers {
            // win
            GameStateState::End(Some(this_player))
        } else if max_layer_won == self.board.layers - 1 {
            // expand prompt
            GameStateState::ExpandPrompt(position.clone())
        } else {
            self.set_domain(position.without(position.depth()-max_layer_won-2).unwrap());
            GameStateState::Play
        };
    }

    pub fn expand_prompt_resolved(&mut self, position:Position, expand:bool) {
        self.other.state = GameStateState::Play;
        if expand { self.board.expand(); }
        self.set_domain(if expand { position } else { position.without(0).unwrap() });
    }

    fn set_domain(&mut self, to:Position) {
        self.board.board.get_or_create_to(&to).get_or_create();
        let (layers, None | Some(PositionError::PositionTooShallow)) = to.get_stack(&self.board.board) else { panic!() };
        let mut directions = vec![];
        for (direction, layer) in layers {
            if layer.value.is_full() { break }
            directions.push(direction);
        }
        self.other.domain = directions.into();
        self.board.board.expand_or_create_to(&self.other.domain).expand_or_create();
    }
}

