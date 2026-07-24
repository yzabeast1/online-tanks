use hyper::{Body, Request};
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerSettings {
    pub bind_ip: String,
    pub port: u16,
    pub connect_link: String,
    #[serde(default = "default_clerk_issuer_url")]
    pub clerk_issuer_url: String,
}

pub fn default_clerk_issuer_url() -> String {
    "https://joint-wahoo-1.clerk.accounts.dev".to_string()
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            bind_ip: "0.0.0.0".to_string(),
            port: 443,
            connect_link: "localhost".to_string(),
            clerk_issuer_url: default_clerk_issuer_url(),
        }
    }
}

pub fn default_can_be_played() -> fn(&Card, &GameState, &Request<Body>) -> bool {
    crate::cards::can_never_play
}

pub fn default_play() -> fn(&Card, &mut GameState, usize, &Request<Body>) {
    crate::cards::stub_play
}

pub fn default_when_done() -> fn(&mut GameState, &ActiveCalculatedShooting, &Request<Body>) {
    crate::cards::stub_when_done
}

pub fn default_active_card() -> Card {
    crate::cards::all_cards()
        .into_iter()
        .next()
        .expect("card list should not be empty")
}

pub fn default_inactive_player_index() -> isize {
    -1
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
    #[serde(default)]
    pub hand: Vec<Card>,
    pub health: isize,
}

impl Player {
    pub fn new(name: String, game_settings: GameSettings) -> Self {
        return Player {
            name,
            hand: Vec::new(),
            health: game_settings.starting_health,
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TurnState {
    pub shooting_card_played: usize,
    #[serde(default)]
    pub distractor_missile_played: usize,
    pub more_ammo_played: usize,
    pub event_card_played: bool,
    pub no_shooting_played: bool,
    #[serde(default)]
    pub shooting_locked: bool,
    pub total_cards_played: usize,
}

impl TurnState {
    pub fn new() -> Self {
        return TurnState {
            shooting_card_played: 0,
            distractor_missile_played: 0,
            more_ammo_played: 0,
            event_card_played: false,
            no_shooting_played: false,
            shooting_locked: false,
            total_cards_played: 0,
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveCalculatedShooting {
    pub owner: String,
    pub turns_remaining: usize,
    #[serde(skip, default = "crate::structs::default_when_done")]
    pub when_done: fn(&mut GameState, &ActiveCalculatedShooting, &Request<Body>),
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
    #[serde(default = "crate::structs::default_inactive_player_index")]
    pub no_shooting_played_by: isize,
    pub no_shooting_card: Option<Card>,
    pub active_calculated_shootings: Vec<ActiveCalculatedShooting>,
    pub active_firing_filters: Vec<ActiveFiringFilter>,
}

impl ActiveCards {
    pub fn new() -> Self {
        return ActiveCards {
            landmine_played_by: -1,
            landmine_card: None,
            no_shooting_played_by: -1,
            no_shooting_card: None,
            active_calculated_shootings: Vec::new(),
            active_firing_filters: Vec::new(),
        };
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingRecycle {
    pub player: String,
    #[serde(default)]
    pub awaiting_discards: Vec<String>,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArmorConfig {
    pub player: String,
    pub enabled: bool,
    pub threshold: isize,
    pub discard_card_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub players: Vec<Player>,
    pub id: String,
    #[serde(default)]
    pub clerk_identities: HashMap<String, String>,
    #[serde(default)]
    pub clerk_pictures: HashMap<String, String>,
    pub current_turn_player: usize,
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub turn_state: TurnState,
    pub active_cards: ActiveCards,
    #[serde(default)]
    pub pending_recycle: Option<PendingRecycle>,
    #[serde(skip)]
    pub death_discard_cards: Vec<Card>,
    pub chat_messages: Vec<ChatMessage>,
    pub game_settings: GameSettings,
    #[serde(default)]
    pub armor_configs: Vec<ArmorConfig>,
    #[serde(default)]
    pub armor_disabled_notifications: Vec<String>,
    #[serde(default)]
    pub ai_count: usize,
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
    #[serde(default)]
    pub clerk_identities: HashMap<String, String>,
    #[serde(default)]
    pub clerk_pictures: HashMap<String, String>,
    pub chat_messages: Vec<ChatMessage>,
    pub game_settings: GameSettings,
}

impl LobbyState {
    pub fn new(host: String, clerk_user_id: Option<String>, clerk_picture: String) -> Self {
        Self::new_with_settings(host, clerk_user_id, clerk_picture, GameSettings::default())
    }

    pub fn new_with_settings(
        host: String,
        clerk_user_id: Option<String>,
        clerk_picture: String,
        game_settings: GameSettings,
    ) -> Self {
        let mut clerk_identities = HashMap::new();
        if let Some(clerk_user_id) = clerk_user_id {
            clerk_identities.insert(host.clone(), clerk_user_id);
        }
        let mut clerk_pictures = HashMap::new();
        if !clerk_picture.is_empty() && clerk_identities.contains_key(&host) {
            clerk_pictures.insert(host.clone(), clerk_picture);
        }

        Self {
            players: vec![host],
            clerk_identities,
            clerk_pictures,
            chat_messages: Vec::new(),
            game_settings,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingSuccessorLobby {
    pub preferred_host: String,
    pub game_settings: GameSettings,
    pub ai_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerState {
    pub lobbies: Mutex<HashMap<String, LobbyState>>,
    pub started_games: Mutex<HashSet<String>>,
    pub games: Mutex<HashMap<String, GameState>>,
    pub joincode_redirects: Mutex<HashMap<String, String>>,
    pub pending_successor_lobbies: Mutex<HashMap<String, PendingSuccessorLobby>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            lobbies: Mutex::new(HashMap::new()),
            started_games: Mutex::new(HashSet::new()),
            games: Mutex::new(HashMap::new()),
            joincode_redirects: Mutex::new(HashMap::new()),
            pending_successor_lobbies: Mutex::new(HashMap::new()),
        }
    }

    pub fn hydrate(&self) {
        let templates = crate::cards::all_cards();
        let mut games = crate::lock_or_recover(&self.games, "games");
        for game in games.values_mut() {
            game.hydrate(&templates);
        }
    }
}

impl GameState {
    pub fn new(
        game_id: String,
        player_names: Vec<String>,
        clerk_identities: HashMap<String, String>,
        clerk_pictures: HashMap<String, String>,
        game_settings: GameSettings,
    ) -> Self {
        let mut players = Vec::new();
        let mut ai_count = 0;
        for name in &player_names {
            players.push(Player::new(name.clone(), game_settings));
            if crate::ai::is_ai_player(name) {
                ai_count += 1;
            }
        }

        return GameState {
            players,
            id: game_id,
            clerk_identities,
            clerk_pictures,
            current_turn_player: 0,
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            turn_state: TurnState::new(),
            active_cards: ActiveCards::new(),
            pending_recycle: None,
            death_discard_cards: Vec::new(),
            chat_messages: Vec::new(),
            game_settings: game_settings,
            armor_configs: Vec::new(),
            armor_disabled_notifications: Vec::new(),
            ai_count,
        };
    }

    pub fn push_server_chat_message(&mut self, message: String) {
        self.chat_messages.push(ChatMessage {
            sender: "server".to_string(),
            message,
            timestamp: crate::current_timestamp(),
        });
    }

    pub fn start_game(&mut self) {
        self.current_turn_player = rand::thread_rng().gen_range(0..self.players.len());
        self.draw_pile = all_cards();
        self.draw_pile.shuffle(&mut thread_rng());
        self.draw_initial_hand();
        self.next_turn();
    }

    pub fn next_turn(&mut self) {
        if self.players.is_empty() || self.alive_player_count() <= 1 {
            return;
        }
        self.turn_state = TurnState::new();
        loop {
            self.current_turn_player = (self.current_turn_player + 1) % self.players.len();
            if self.players[self.current_turn_player].health > 0 {
                break;
            }
        }
        for calculated_shooting in self.active_cards.active_calculated_shootings.iter_mut() {
            if calculated_shooting.owner == self.players[self.current_turn_player].name {
                calculated_shooting.turns_remaining =
                    calculated_shooting.turns_remaining.saturating_sub(1);
            }
        }
        if self.active_cards.no_shooting_played_by == self.current_turn_player as isize {
            if let Some(card) = self.active_cards.no_shooting_card.take() {
                self.discard_pile.push(card);
                self.push_server_chat_message("No Shooting went out of play".to_string());
            }
            self.active_cards.no_shooting_played_by = -1;
        }
        let current_player_name = self.players[self.current_turn_player].name.clone();
        let mut active_firing_filters =
            std::mem::take(&mut self.active_cards.active_firing_filters);
        let mut discarded_firing_filter_names = Vec::new();
        active_firing_filters.retain(|filter| {
            if filter.owner == current_player_name {
                self.discard_pile.push(filter.card_played.clone());
                discarded_firing_filter_names.push(filter.card_played.name.clone());
                return false;
            } else {
                return true;
            }
        });
        self.active_cards.active_firing_filters = active_firing_filters;
        for discarded_filter_name in discarded_firing_filter_names {
            self.push_server_chat_message(format!("{} went out of play", discarded_filter_name));
        }
        self.draw_card(self.current_turn_player);
        if self.active_cards.landmine_played_by != -1 && rand::thread_rng().gen_bool(0.5) {
            let hit_player_name = self.players[self.current_turn_player].name.clone();
            let landmine_owner = self.active_cards.landmine_played_by;
            self.active_cards.landmine_played_by = -1;
            if let Some(card) = self.active_cards.landmine_card.take() {
                self.discard_pile.push(card);
                self.push_server_chat_message("Landmine went out of play".to_string());
            }
            self.push_server_chat_message(format!("Landmine hit {}", hit_player_name));
            let self_inflicted = landmine_owner == self.current_turn_player as isize;
            self.damage_player(self.current_turn_player, 6, self_inflicted);
        }
    }

    pub fn damage_player(&mut self, player_index: usize, damage: isize, self_inflicted: bool) {
        if player_index >= self.players.len() {
            return;
        }
        if self.players[player_index].health <= 0 {
            return;
        }

        let player_name = self.players[player_index].name.clone();
        let mut effective_damage = damage;

        // Armor does not activate on self-inflicted damage
        if !self_inflicted {
            effective_damage = self.apply_armor_reductions(player_index, effective_damage);
        }

        self.players[player_index].health -= effective_damage;
        let has_last_stand = self.players[player_index]
            .hand
            .iter()
            .position(|card| card.id == "last-stand");

        if self.players[player_index].health < 1 {
            if let Some(last_stand_index) = has_last_stand {
                self.players[player_index].health = 1;
                self.players[player_index].hand.remove(last_stand_index);
                self.push_server_chat_message(format!("Last Stand activated for {}", player_name));
            } else {
                if self.game_settings.revive_others_with_heal {
                    self.push_server_chat_message(format!("{} died", player_name));
                    self.players[player_index].health = 0;
                    self.discard_active_cards_for_player(&player_name);
                    self.armor_configs.retain(|c| c.player != player_name);
                    if self.active_cards.no_shooting_played_by == player_index as isize {
                        self.active_cards.no_shooting_played_by = -1;
                        if let Some(card) = self.active_cards.no_shooting_card.take() {
                            self.discard_pile.push(card);
                        }
                    }
                    if self.current_turn_player == player_index && self.alive_player_count() > 1 {
                        self.next_turn();
                    }
                } else {
                    let discarded_hand = self.players[player_index].hand.clone();
                    let discarded_hand_names = discarded_hand
                        .iter()
                        .map(|card| card.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.death_discard_cards.extend(discarded_hand);
                    if discarded_hand_names.is_empty() {
                        self.push_server_chat_message(format!("{} died", player_name));
                    } else {
                        self.push_server_chat_message(format!(
                            "{} died and discarded {}",
                            player_name, discarded_hand_names
                        ));
                    }
                    self.remove_player(player_index, true);
                }
            }
        }
    }

    pub fn heal_player(&mut self, player_index: usize, heal_ammount: isize) {
        if player_index >= self.players.len() {
            return;
        }
        self.players[player_index].health += heal_ammount;
        if self.players[player_index].health > self.game_settings.max_health {
            self.players[player_index].health = self.game_settings.max_health;
        }
    }

    pub fn alive_player_count(&self) -> usize {
        self.players
            .iter()
            .filter(|player| player.health > 0)
            .count()
    }

    pub fn validate_armor_configs(&mut self) {
        let mut i = 0;
        while i < self.armor_configs.len() {
            let config = &self.armor_configs[i];
            let player = self.players.iter().find(|p| p.name == config.player);
            let valid = if let Some(player) = player {
                let armor_count = player.hand.iter().filter(|c| c.id == "armor").count();
                let configs_through_this_one = self.armor_configs[..=i]
                    .iter()
                    .filter(|other| other.player == config.player)
                    .count();
                let armor_discards_through_this_one = self.armor_configs[..=i]
                    .iter()
                    .filter(|other| {
                        other.player == config.player && other.discard_card_id == "armor"
                    })
                    .count();
                let has_discard = player.hand.iter().any(|c| c.id == config.discard_card_id);
                player.health > 0
                    && armor_count >= configs_through_this_one + armor_discards_through_this_one
                    && has_discard
            } else {
                false
            };
            if !valid {
                let config = self.armor_configs.remove(i);
                if config.enabled {
                    // Notification is private: included in next game state poll for this player
                    self.armor_disabled_notifications
                        .push(config.player.clone());
                }
            } else {
                i += 1;
            }
        }
    }

    fn try_activate_single_armor(&mut self, player_index: usize, config_index: usize) -> bool {
        let discard_id = self.armor_configs[config_index].discard_card_id.clone();
        let armor_index = self.players[player_index]
            .hand
            .iter()
            .position(|card| card.id == "armor");
        let discard_index = self.players[player_index]
            .hand
            .iter()
            .position(|card| card.id == discard_id);

        let (Some(armor_index), Some(discard_index)) = (armor_index, discard_index) else {
            return false;
        };

        let (first, second) = if armor_index > discard_index {
            (armor_index, discard_index)
        } else if armor_index < discard_index {
            (discard_index, armor_index)
        } else {
            let second_armor = self.players[player_index]
                .hand
                .iter()
                .enumerate()
                .position(|(index, card)| index != armor_index && card.id == "armor");

            match second_armor {
                Some(second_armor) if second_armor > armor_index => (second_armor, armor_index),
                Some(second_armor) => (armor_index, second_armor),
                None => return false,
            }
        };

        let removed_first = self.players[player_index].hand.remove(first);
        self.discard_pile.push(removed_first);
        let removed_second = self.players[player_index].hand.remove(second);
        self.discard_pile.push(removed_second);
        self.armor_configs.remove(config_index);
        true
    }

    fn apply_armor_reductions(&mut self, player_index: usize, damage: isize) -> isize {
        let player_name = self.players[player_index].name.clone();
        let mut remaining_damage = damage;

        loop {
            let config_index = self
                .armor_configs
                .iter()
                .position(|config| config.enabled && config.player == player_name);
            let Some(config_index) = config_index else {
                break;
            };

            if remaining_damage < self.armor_configs[config_index].threshold {
                break;
            }

            if !self.try_activate_single_armor(player_index, config_index) {
                break;
            }

            let new_damage = (remaining_damage - 3).max(0);
            self.push_server_chat_message(format!(
                "{} activated Armor reducing {} damage to {}",
                player_name, remaining_damage, new_damage
            ));
            remaining_damage = new_damage;

            if remaining_damage == 0 {
                break;
            }
        }

        self.validate_armor_configs();
        remaining_damage
    }

    fn discard_active_cards_for_player(&mut self, player_name: &str) {
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
    }

    pub fn remove_player(&mut self, player_index: usize, advance_turn_if_current: bool) -> bool {
        if player_index >= self.players.len() {
            return self.players.is_empty();
        }

        let player_name = self.players[player_index].name.clone();
        let was_their_turn = self.current_turn_player == player_index;

        self.discard_active_cards_for_player(&player_name);
        self.armor_configs.retain(|c| c.player != player_name);

        if self.active_cards.no_shooting_played_by == player_index as isize {
            self.active_cards.no_shooting_played_by = if self.players.len() > 1 {
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
            self.active_cards.no_shooting_played_by = -1;
            return true;
        }

        if self.active_cards.no_shooting_played_by > player_index as isize {
            self.active_cards.no_shooting_played_by -= 1;
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
            for _ in 0..self.game_settings.starting_hand_size {
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
        if let Some(pending_recycle) = &mut self.pending_recycle {
            for card in &mut pending_recycle.cards {
                if let Some(template) = template_cards.iter().find(|t| t.id == card.id) {
                    card.can_be_played = template.can_be_played;
                    card.play = template.play;
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct GameSettings {
    pub starting_hand_size: usize,
    pub starting_health: isize,
    pub max_health: isize,
    pub play_heal_on_others: bool,
    pub revive_others_with_heal: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            starting_hand_size: 4,
            starting_health: 10,
            max_health: 10,
            play_heal_on_others: false,
            revive_others_with_heal: false,
        }
    }
}
