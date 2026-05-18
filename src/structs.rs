use hyper::{Body, Request};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Mutex;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardType {
    Support,
    Plus,
    Shooting(ShootingType),
    Event,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShootingType {
    Quick,
    Calculated,
    Boom,
}

impl CardType {
    pub fn is_shooting(&self) -> bool {
        matches!(self, CardType::Shooting(_))
    }

    pub fn shooting_type(&self) -> Option<ShootingType> {
        match self {
            CardType::Shooting(shooting_type) => Some(*shooting_type),
            _ => None,
        }
    }

    pub fn is_shooting_type(&self, shooting_type: ShootingType) -> bool {
        self.shooting_type() == Some(shooting_type)
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    pub name: String,
    pub id: String,
    pub card_type: CardType,
    pub can_be_played: fn(&Card, &GameState, &Request<Body>) -> bool,
    pub play: fn(&Card, &mut GameState, usize),
    pub count:usize,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub name: String,
    pub id: usize,
    pub hand: Vec<Card>,
    pub health: usize,
}

impl Player {
    pub fn new(name: String, id: usize) -> Self {
        return Player {
            name,
            id,
            hand: Vec::new(),
            health: 10,
        };
    }

    pub fn draw_card(&mut self, card: Card) {
        self.hand.push(card);
    }
}

pub struct TurnState {
    pub shooting_card_played: usize,
    pub more_ammo_played: bool,
    pub event_card_played: bool,
    pub no_shooting_played: bool,
    pub total_cards_played: usize,
}

impl TurnState {
    pub fn new() -> Self {
        return TurnState {
            shooting_card_played: 0,
            more_ammo_played: false,
            event_card_played: false,
            no_shooting_played: false,
            total_cards_played: 0,
        };
    }
}

pub struct active_calculated_shooting {
    pub owner: usize,
    pub turns_remaining: usize,
    pub when_done: fn(),
    pub card_played: Card,
}
pub struct active_fiting_filter {
    pub owner: usize,
    pub filter_type: ShootingType,
}

pub struct ActiveCards {
    pub landmine_played_by: isize,
    pub active_calculated_shootings: Vec<active_calculated_shooting>,
    pub active_firing_filters: Vec<active_fiting_filter>,
}

impl ActiveCards {
    pub fn new() -> Self {
        return ActiveCards {
            landmine_played_by: -1,
            active_calculated_shootings: Vec::new(),
            active_firing_filters: Vec::new(),
        };
    }
}

pub struct GameState {
    pub players: Vec<Player>,
    pub id: String,
    pub current_turn_player: usize,
    pub draw_pile: VecDeque<Card>,
    pub discard_pile: VecDeque<Card>,
    pub no_shooting_played_by: isize,
    pub turn_state: TurnState,
    pub active_cards: ActiveCards,
}

#[derive(Debug)]
pub struct ServerState {
    pub lobbies: Mutex<HashMap<String, Vec<String>>>,
    pub started_games: Mutex<HashSet<String>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            lobbies: Mutex::new(HashMap::new()),
            started_games: Mutex::new(HashSet::new()),
        }
    }
}

impl GameState {
    pub fn new(game_id: String, num_players: usize) -> Self {
        let mut players = Vec::new();
        for i in 0..num_players {
            players.push(Player::new(format!("Player {}", i + 1), i));
        }

        return GameState {
            players,
            id: game_id,
            current_turn_player: 0,
            draw_pile: VecDeque::new(),
            discard_pile: VecDeque::new(),
            no_shooting_played_by: -1,
            turn_state: TurnState::new(),
            active_cards: ActiveCards::new(),
        };
    }

    pub fn next_turn(&mut self) {
        self.turn_state = TurnState::new();
        self.current_turn_player = (self.current_turn_player + 1) % self.players.len();
    }

    pub fn draw_card(&mut self) -> Option<Card> {
        return self.draw_pile.pop_front();
    }

    pub fn player_by_id(&self, player_id: usize) -> Option<&Player> {
        self.players.iter().find(|player| player.id == player_id)
    }

    pub fn player_by_id_mut(&mut self, player_id: usize) -> Option<&mut Player> {
        self.players
            .iter_mut()
            .find(|player| player.id == player_id)
    }

    pub fn draw_initial_hand(&mut self) {
        for i in 0..self.players.len() {
            for _ in 0..4 {
                if let Some(card) = self.draw_card() {
                    self.players[i].hand.push(card);
                }
            }
        }
    }
}
