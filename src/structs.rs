use hyper::{Body, Request};
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::cards::all_cards;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum CardType {
    Support,
    Plus,
    Shooting(ShootingType),
    Event,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub name: String,
    pub id: String,
    pub card_type: CardType,
    #[serde(skip)]
    pub can_be_played: fn(&Card, &GameState, &Request<Body>) -> bool,
    #[serde(skip)]
    pub play: fn(&Card, &mut GameState, usize),
    #[serde(skip)]
    pub count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Player {
    pub name: String,
    pub id: usize,
    pub hand: Vec<Card>,
    pub health: isize,
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
    pub fn take_damage(&mut self, damage: isize) {
        self.health -= damage;
        let has_last_stand = self.hand.iter().position(|card| card.id == "last-stand");
        if self.health < 1 {
            if has_last_stand.is_some() {
                self.health = 1;
                self.hand.remove(has_last_stand.unwrap());
            }
            else {
                
            }
        }
    }
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ActiveCalculatedShooting {
    pub owner: usize,
    pub turns_remaining: usize,
    #[serde(skip)]
    pub when_done: fn(),
    pub card_played: Card,
}
#[derive(Debug, Serialize)]
pub struct ActiveFitingFilter {
    pub owner: usize,
    pub filter_type: ShootingType,
}

#[derive(Debug, Serialize)]
pub struct ActiveCards {
    pub landmine_played_by: isize,
    pub active_calculated_shootings: Vec<ActiveCalculatedShooting>,
    pub active_firing_filters: Vec<ActiveFitingFilter>,
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

#[derive(Debug, Serialize)]
pub struct GameState {
    pub players: Vec<Player>,
    pub id: String,
    pub current_turn_player: usize,
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub no_shooting_played_by: isize,
    pub turn_state: TurnState,
    pub active_cards: ActiveCards,
    pub chat_messages: Vec<ChatMessage>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
    pub timestamp: usize,
}

#[derive(Debug)]
pub struct LobbyState {
    pub players: Vec<String>,
    pub chat_messages: Vec<ChatMessage>,
}

impl LobbyState {
    pub fn new(host: String) -> Self {
        Self {
            players: vec![host],
            chat_messages: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ServerState {
    pub lobbies: Mutex<HashMap<String, LobbyState>>,
    pub started_games: Mutex<HashSet<String>>,
    pub games: Mutex<HashMap<String, GameState>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            lobbies: Mutex::new(HashMap::new()),
            started_games: Mutex::new(HashSet::new()),
            games: Mutex::new(HashMap::new()),
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
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            no_shooting_played_by: -1,
            turn_state: TurnState::new(),
            active_cards: ActiveCards::new(),
            chat_messages: Vec::new(),
        };
    }

    pub fn start_game(&mut self) {
        self.current_turn_player = rand::thread_rng().gen_range(0..self.players.len());
        self.draw_pile = all_cards();
        self.draw_initial_hand();
        self.next_turn();
    }

    pub fn next_turn(&mut self) {
        self.turn_state = TurnState::new();
        for calculated_shooting in self.active_cards.active_calculated_shootings.iter_mut() {
            if calculated_shooting.owner == self.players[self.current_turn_player].id {
                calculated_shooting.turns_remaining -= 1;
            }
        }
        self.current_turn_player = (self.current_turn_player + 1) % self.players.len();
        self.draw_card(self.current_turn_player);
        if self.active_cards.landmine_played_by != -1 && rand::thread_rng().gen_bool(0.5) {
            self.players[self.current_turn_player].take_damage(6);
            self.active_cards.landmine_played_by = -1;
        }
    }

    pub fn draw_card(&mut self, player_index: usize) {
        if self.draw_pile.is_empty() {
            self.discard_pile.shuffle(&mut thread_rng());
            self.draw_pile = std::mem::take(&mut self.discard_pile);
            self.discard_pile = Vec::new();
        }
        if let Some(card) = self.draw_pile.pop() {
            self.players[player_index].hand.push(card);
        }
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
                self.draw_card(i);
            }
        }
    }
}
