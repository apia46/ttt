use educe::Educe;
use macros::parse_color;
use vello::{kurbo::{Point, Size}, peniko::Color};

use crate::{board::{BoardState, LayerState, Position, PositionError, Value}, button::{self, simple::{NewSimpleButtonArgs, SimpleButton}}, space::{Space, anchor::{Anchor, AnchorContext, BottomMiddle, BottomRight}}};

#[derive(Debug, Clone, Copy)]
pub enum Player { X, O }
impl Player {
    #[must_use]
    pub fn to_value(self) -> Value {
        match self {
            Player::X => Value::X,
            Player::O => Value::O,
        }
    }

    pub fn color(player:Option<Player>) -> Color {
        match player {
            None => parse_color!("#ff00ff"),
            Some(Player::X) => parse_color!("#00ffff"),
            Some(Player::O) => parse_color!("#ff0066"),
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
#[derive(Educe)]
#[educe(Default)]
pub struct GameOtherState {
    #[educe(Default=Player::X)]
    pub turn: Player,
    pub domain: Position,
    pub state: GameStateState,
    pub expand_denied:bool,
    #[educe(Default=home_button())]
    pub home_button:SimpleButton<AnchorContext<BottomRight>>,
}

#[derive(Default)]
pub enum GameStateState {
    #[default] Play,
    ExpandPrompt(ExpandPrompt),
    End(Option<Player>),
}

pub struct ExpandPrompt {
    pub expand_position:Position,
    pub yes_button:SimpleButton<AnchorContext<BottomMiddle>>,
    pub no_button:SimpleButton<AnchorContext<BottomMiddle>>,
}

impl ExpandPrompt {
    fn new(expand_position:Position) -> Self {
        ExpandPrompt { expand_position, yes_button: yes_button(), no_button: no_button() }
    }
}

const BUTTON_STYLE:button::simple::Style = button::simple::Style {
    fill_color: parse_color!("#223344"),
    hover_fill_color: parse_color!("#476685"),
    border_color: parse_color!("#668899"),
    text_color: parse_color!("#ffffff"),
    font_size: 20.,
};

fn home_button() -> SimpleButton<AnchorContext<BottomRight>> {
    SimpleButton::new(NewSimpleButtonArgs {
        rect: Anchor::from_center_size(Point::new(-30., -30.).wrapped(), Size::new(40., 40.).wrapped()),
        text: "Home".into(), style: &BUTTON_STYLE, disabled: false,
    })
}

fn yes_button() -> SimpleButton<AnchorContext<BottomMiddle>> {
    const STYLE:button::simple::Style = button::simple::Style { border_color: parse_color!("#66997a"), ..BUTTON_STYLE };
    SimpleButton::new(NewSimpleButtonArgs {
        rect: Anchor::from_center_size(Point::new(-50., -60.).wrapped(), Size::new(50., 20.).wrapped()),
        text: "Yes".into(), style: &STYLE, disabled: false,
    })
}

fn no_button() -> SimpleButton<AnchorContext<BottomMiddle>> {
    const STYLE:button::simple::Style = button::simple::Style { border_color: parse_color!("#854747"), ..BUTTON_STYLE };
    SimpleButton::new(NewSimpleButtonArgs {
        rect: Anchor::from_center_size(Point::new(50., -60.).wrapped(), Size::new(50., 20.).wrapped()),
        text: "No".into(), style: &STYLE, disabled: false,
    })
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
            if self.other.expand_denied {
                self.set_domain(position.without(0).unwrap());
                GameStateState::Play
            } else {
                GameStateState::ExpandPrompt(ExpandPrompt::new(position.clone()))
            }
        } else {
            self.set_domain(position.without(position.depth()-max_layer_won-2).unwrap());
            GameStateState::Play
        };
    }

    pub fn expand_prompt_resolved(&mut self, position:Position, expand:bool) {
        self.other.state = GameStateState::Play;
        if expand { self.board.expand(); }
        else { self.other.expand_denied = true; }
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

