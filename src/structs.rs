use hyper::{Body, Request};
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub fn default_can_be_played() -> fn(&Card, &GameState, &Request<Body>) -> bool {
    crate::cards::can_never_play
}

pub fn default_play() -> fn(&Card, &mut GameState, usize, &Request<Body>) {
    crate::cards::stub_play
}

pub fn default_when_done() -> fn() {
    crate::cards::stub_when_done
}

pub fn default_active_card() -> Card {
    crate::cards::all_cards()
        .into_iter()
        .next()
        .expect("card list should not be empty")
}

use crate::cards::all_cards;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CardType {
    Support,
    Plus,
    Shooting(ShootingType),
    Event,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub id: String,
    pub card_type: CardType,
    #[serde(skip, default = "crate::structs::default_can_be_played")]
    pub can_be_played: fn(&Card, &GameState, &Request<Body>) -> bool,
    #[serde(skip, default = "crate::structs::default_play")]
    pub play: fn(&Card, &mut GameState, usize, &Request<Body>),
    #[serde(skip)]
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    #[serde(skip)]
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TurnState {
    pub shooting_card_played: usize,
    pub more_ammo_played: usize,
    pub event_card_played: bool,
    pub no_shooting_played: bool,
    pub total_cards_played: usize,
}

impl TurnState {
    pub fn new() -> Self {
        return TurnState {
            shooting_card_played: 0,
            more_ammo_played: 0,
            event_card_played: false,
            no_shooting_played: false,
            total_cards_played: 0,
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveCalculatedShooting {
    pub owner: String,
    pub turns_remaining: usize,
    #[serde(skip, default = "crate::structs::default_when_done")]
    pub when_done: fn(),
    pub card_played: Card,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveFiringFilter {
    pub owner: String,
    pub filter_type: ShootingType,
    #[serde(default = "crate::structs::default_active_card")]
    pub card_played: Card,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveCards {
    pub landmine_played_by: isize,
    pub landmine_card: Option<Card>,
    pub no_shooting_card: Option<Card>,
    pub active_calculated_shootings: Vec<ActiveCalculatedShooting>,
    pub active_firing_filters: Vec<ActiveFiringFilter>,
}

impl ActiveCards {
    pub fn new() -> Self {
        return ActiveCards {
            landmine_played_by: -1,
            landmine_card: None,
            no_shooting_card: None,
            active_calculated_shootings: Vec::new(),
            active_firing_filters: Vec::new(),
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
    pub timestamp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn hydrate(&self) {
        let templates = crate::cards::all_cards();
        let mut games = self.games.lock().unwrap();
        for game in games.values_mut() {
            game.hydrate(&templates);
        }
    }
}

impl GameState {
    pub fn new(game_id: String, player_names: Vec<String>) -> Self {
        let mut players = Vec::new();
        for (i, name) in player_names.into_iter().enumerate() {
            players.push(Player::new(name, i));
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
        self.draw_pile.shuffle(&mut thread_rng());
        self.draw_initial_hand();
        self.next_turn();
    }

    pub fn next_turn(&mut self) {
        if self.players.is_empty() {
            return;
        }
        self.turn_state = TurnState::new();
        for calculated_shooting in self.active_cards.active_calculated_shootings.iter_mut() {
            if calculated_shooting.owner == self.players[self.current_turn_player].name {
                calculated_shooting.turns_remaining =
                    calculated_shooting.turns_remaining.saturating_sub(1);
            }
        }
        self.current_turn_player = (self.current_turn_player + 1) % self.players.len();
        if self.no_shooting_played_by == self.current_turn_player as isize {
            if let Some(card) = self.active_cards.no_shooting_card.take() {
                self.discard_pile.push(card);
            }
            self.no_shooting_played_by = -1;
        }
        let current_player_name = self.players[self.current_turn_player].name.clone();
        let mut active_firing_filters =
            std::mem::take(&mut self.active_cards.active_firing_filters);
        active_firing_filters.retain(|filter| {
            if filter.owner == current_player_name {
                self.discard_pile.push(filter.card_played.clone());
                return false;
            } else {
                return true;
            }
        });
        self.active_cards.active_firing_filters = active_firing_filters;
        self.draw_card(self.current_turn_player);
        if self.active_cards.landmine_played_by != -1 && rand::thread_rng().gen_bool(0.5) {
            self.active_cards.landmine_played_by = -1;
            if let Some(card) = self.active_cards.landmine_card.take() {
                self.discard_pile.push(card);
            }
            self.damage_player(self.current_turn_player, 6);
        }
    }

    pub fn damage_player(&mut self, player_index: usize, damage: isize) {
        if player_index >= self.players.len() {
            return;
        }

        self.players[player_index].health -= damage;
        let has_last_stand = self.players[player_index]
            .hand
            .iter()
            .position(|card| card.id == "last-stand");

        if self.players[player_index].health < 1 {
            if let Some(last_stand_index) = has_last_stand {
                self.players[player_index].health = 1;
                self.players[player_index].hand.remove(last_stand_index);
            } else {
                self.remove_player(player_index, true);
            }
        }
    }

    pub fn remove_player(&mut self, player_index: usize, advance_turn_if_current: bool) -> bool {
        if player_index >= self.players.len() {
            return self.players.is_empty();
        }

        let player_name = self.players[player_index].name.clone();
        let was_their_turn = self.current_turn_player == player_index;

        let mut active_firing_filters =
            std::mem::take(&mut self.active_cards.active_firing_filters);
        active_firing_filters.retain(|filter| {
            if filter.owner == player_name {
                self.discard_pile.push(filter.card_played.clone());
                false
            } else {
                true
            }
        });
        self.active_cards.active_firing_filters = active_firing_filters;

        let mut active_calculated_shootings =
            std::mem::take(&mut self.active_cards.active_calculated_shootings);
        active_calculated_shootings.retain(|shooting| {
            if shooting.owner == player_name {
                self.discard_pile.push(shooting.card_played.clone());
                false
            } else {
                true
            }
        });
        self.active_cards.active_calculated_shootings = active_calculated_shootings;

        if self.no_shooting_played_by == player_index as isize {
            self.no_shooting_played_by = if self.players.len() > 1 {
                ((player_index + 1) % self.players.len()) as isize
            } else {
                -1
            };
        }

        let hand = std::mem::take(&mut self.players[player_index].hand);
        self.discard_pile.extend(hand);
        self.players.remove(player_index);

        if self.players.is_empty() {
            self.current_turn_player = 0;
            self.no_shooting_played_by = -1;
            return true;
        }

        if self.no_shooting_played_by > player_index as isize {
            self.no_shooting_played_by -= 1;
        }
        if self.current_turn_player > player_index {
            self.current_turn_player -= 1;
        } else if self.current_turn_player >= self.players.len() {
            self.current_turn_player = 0;
        }

        if was_their_turn && advance_turn_if_current {
            self.current_turn_player = if self.current_turn_player == 0 {
                self.players.len() - 1
            } else {
                self.current_turn_player - 1
            };
            self.next_turn();
        }

        false
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

    pub fn player_by_name(&self, name: &str) -> Option<&Player> {
        self.players.iter().find(|player| player.name == name)
    }

    pub fn player_index_from_name(&self, name: &str) -> Option<usize> {
        self.players.iter().position(|player| player.name == name)
    }

    pub fn player_by_name_mut(&mut self, name: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|player| player.name == name)
    }

    pub fn draw_initial_hand(&mut self) {
        for i in 0..self.players.len() {
            for _ in 0..4 {
                self.draw_card(i);
            }
        }
    }

    pub fn hydrate(&mut self, template_cards: &[Card]) {
        for card in &mut self.draw_pile {
            if let Some(template) = template_cards.iter().find(|t| t.id == card.id) {
                card.can_be_played = template.can_be_played;
                card.play = template.play;
            }
        }
        for card in &mut self.discard_pile {
            if let Some(template) = template_cards.iter().find(|t| t.id == card.id) {
                card.can_be_played = template.can_be_played;
                card.play = template.play;
            }
        }
        for player in &mut self.players {
            for card in &mut player.hand {
                if let Some(template) = template_cards.iter().find(|t| t.id == card.id) {
                    card.can_be_played = template.can_be_played;
                    card.play = template.play;
                }
            }
        }
        for active in &mut self.active_cards.active_calculated_shootings {
            if let Some(template) = template_cards
                .iter()
                .find(|t| t.id == active.card_played.id)
            {
                active.card_played.can_be_played = template.can_be_played;
                active.card_played.play = template.play;
            }
        }
        for active in &mut self.active_cards.active_firing_filters {
            if let Some(template) = template_cards
                .iter()
                .find(|t| t.id == active.card_played.id)
            {
                active.card_played.can_be_played = template.can_be_played;
                active.card_played.play = template.play;
            }
        }
        if let Some(card) = &mut self.active_cards.landmine_card {
            if let Some(template) = template_cards.iter().find(|t| t.id == card.id) {
                card.can_be_played = template.can_be_played;
                card.play = template.play;
            }
        }
        if let Some(card) = &mut self.active_cards.no_shooting_card {
            if let Some(template) = template_cards.iter().find(|t| t.id == card.id) {
                card.can_be_played = template.can_be_played;
                card.play = template.play;
            }
        }
    }
}
