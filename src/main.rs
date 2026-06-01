mod cards;
mod structs;
use structs::*;

use std::{
    convert::Infallible,
    fs::File,
    io::BufReader,
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard},
};

use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, StatusCode};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::cards::all_cards;

const WEBSITE_DIR: &str = "./website";
const IMAGES_DIR: &str = "./images";
const SHOO_BASE_URL: &str = "https://shoo.dev";
const SHOO_ISSUER: &str = "https://shoo.dev";

async fn serve_file(path: &str, content_type: &str) -> Result<Response<Body>, Infallible> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", content_type)
            .body(Body::from(bytes))
            .unwrap()),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    }
}

fn image_content_type(path: &str) -> &'static str {
    match std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

async fn serve_image(path: &str) -> Result<Response<Body>, Infallible> {
    serve_file(path, image_content_type(path)).await
}

async fn serve_lobby_js(settings: &ServerSettings) -> Result<Response<Body>, Infallible> {
    let path = format!("{}/lobby.js", WEBSITE_DIR);
    match tokio::fs::read_to_string(&path).await {
        Ok(contents) => {
            let serverip = if settings.port == 443 {
                settings.connect_link.clone()
            } else {
                format!("{}:{}", settings.connect_link, settings.port)
            };
            let contents = contents
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with("var serverip") {
                        format!("var serverip = '{}'", serverip)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/javascript")
                .body(Body::from(contents))
                .unwrap())
        }
        Err(_) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    }
}

fn header_value(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

fn lock_or_recover<'a, T>(mutex: &'a Mutex<T>, name: &str) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("recovering poisoned {} mutex", name);
            poisoned.into_inner()
        }
    }
}

fn text_response(status: StatusCode, body: impl Into<Body>) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .header("access-control-allow-origin", "*")
        .header("access-control-expose-headers", "joincode")
        .body(body.into())
        .unwrap()
}

fn json_response(status: StatusCode, body: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .body(Body::from(body))
        .unwrap()
}

fn current_timestamp() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as usize,
        Err(_) => 0,
    }
}

fn card_stays_active(card: &Card) -> bool {
    matches!(card.card_type, CardType::Shooting(ShootingType::Calculated))
        || card.id == "firing-filter"
        || card.id == "landmine"
        || card.id == "no-shooting"
}

fn card_log_value(card: &Card) -> serde_json::Value {
    serde_json::json!({
        "id": card.id,
        "name": card.name,
        "card_type": format!("{:?}", card.card_type),
    })
}

fn card_target_damage(card: &Card) -> Option<usize> {
    match card.id.as_str() {
        "crack" => Some(2),
        "dent" => Some(1),
        "stolen-parts" => Some(2),
        "big-bomb" => Some(6),
        "small-bomb" => Some(3),
        _ => None,
    }
}

fn damage_amount(before: isize, after: isize) -> Option<usize> {
    (before > after).then_some((before - after) as usize)
}

fn load_server_settings(settings_file_path: &str) -> ServerSettings {
    if std::path::Path::new(settings_file_path).exists() {
        println!("Loading server settings from {}", settings_file_path);
        let data = std::fs::read_to_string(settings_file_path)
            .expect("Failed to read server_settings.json");
        serde_json::from_str(&data).expect("Failed to parse server_settings.json")
    } else {
        let settings = ServerSettings::default();
        let json = serde_json::to_string_pretty(&settings)
            .expect("Failed to serialize default server settings");
        std::fs::write(settings_file_path, json)
            .expect("Failed to create default server_settings.json");
        println!("Created default server settings at {}", settings_file_path);
        settings
    }
}

fn server_connect_url(settings: &ServerSettings) -> String {
    if settings.port == 443 {
        format!("https://{}", settings.connect_link)
    } else {
        format!("https://{}:{}", settings.connect_link, settings.port)
    }
}

fn format_discarded_cards(discarded_cards: &[Card]) -> String {
    discarded_cards
        .iter()
        .map(|card| card.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_targeted_healing_card(card: &Card) -> bool {
    matches!(card.id.as_str(), "repair" | "repair-kit" | "new-model")
}

fn format_play_event_message(
    player: &str,
    card: &Card,
    target_name: Option<&str>,
    firing_filter_type: Option<&str>,
    discarded_cards: &[Card],
    current_player_health_before: isize,
    current_player_health_after: isize,
) -> String {
    let mut message = format!("{} played {}", player, card.name);

    if let Some(target_name) = target_name {
        let target_preposition = if is_targeted_healing_card(card) {
            "on"
        } else {
            "against"
        };
        message.push_str(&format!(" {} {}", target_preposition, target_name));
        if let Some(target_damage) = card_target_damage(card) {
            message.push_str(&format!(" dealing {} damage", target_damage));
        }
    }

    if !discarded_cards.is_empty() {
        message.push_str(&format!(
            " and discarded {}",
            format_discarded_cards(discarded_cards)
        ));
    }

    if let Some(self_damage) =
        damage_amount(current_player_health_before, current_player_health_after)
    {
        message.push_str(&format!(" and took {} damage", self_damage));
    }

    if card.id == "firing-filter" {
        if let Some(firing_filter_type) = firing_filter_type {
            message.push_str(&format!(" filtering {}", firing_filter_type));
        }
    }

    message
}

fn resolved_hand_card(hand: &[Card], value: Option<String>) -> Option<Card> {
    let value = value?;
    if let Ok(index) = value.parse::<usize>() {
        return hand.get(index).cloned();
    }

    hand.iter().find(|card| card.id == value).cloned()
}

fn selected_card_value(
    hand: &[Card],
    name: &str,
    req: &Request<Body>,
) -> Option<serde_json::Value> {
    let raw = header_value(req, name)?;
    let resolved_card = resolved_hand_card(hand, Some(raw.clone()));
    Some(serde_json::json!({
        "raw": raw,
        "resolved_card": resolved_card.as_ref().map(card_log_value),
    }))
}

fn removed_card(before: &[Card], after: &[Card]) -> Option<Card> {
    let mut after_cards = after.to_vec();
    for card in before {
        if let Some(position) = after_cards.iter().position(|candidate| {
            candidate.id == card.id
                && candidate.name == card.name
                && candidate.card_type == card.card_type
        }) {
            after_cards.remove(position);
        } else {
            return Some(card.clone());
        }
    }
    None
}

fn log_play_event(
    game: &mut GameState,
    joincode: &str,
    username: &str,
    card: &Card,
    req: &Request<Body>,
    hand_before: &[Card],
    status: &str,
    discarded_cards: &[Card],
    blocked_filter: Option<&Card>,
    stays_active: bool,
    current_player_health_before: isize,
    current_player_health_after: isize,
    target_name: Option<String>,
    target_health_before: Option<isize>,
    target_hand_before: Option<&[Card]>,
    target_hand_after: Option<&[Card]>,
) {
    let mut payload = serde_json::Map::new();
    payload.insert("event".to_string(), serde_json::json!("play_card"));
    payload.insert("joincode".to_string(), serde_json::json!(joincode));
    payload.insert("player".to_string(), serde_json::json!(username));
    payload.insert("card".to_string(), card_log_value(card));
    payload.insert("status".to_string(), serde_json::json!(status));

    if let Some(target) = target_name.clone() {
        payload.insert("target".to_string(), serde_json::json!(target));
    }

    if let Some(target_health_before) = target_health_before {
        payload.insert(
            "target_health_before".to_string(),
            serde_json::json!(target_health_before),
        );
    }

    if let Some(target_hand_before) = target_hand_before {
        payload.insert(
            "target_hand_before_count".to_string(),
            serde_json::json!(target_hand_before.len()),
        );
    }

    if let Some(value) = selected_card_value(hand_before, "discardcard", req) {
        payload.insert("discardcard".to_string(), value);
    }

    if let Some(value) = selected_card_value(hand_before, "discardcardtwo", req) {
        payload.insert("discardcardtwo".to_string(), value);
    }

    if let Some(value) = selected_card_value(hand_before, "queuedcard", req) {
        payload.insert("queuedcard".to_string(), value);
    }

    if let Some(firing_filter_type) = header_value(req, "firing_filter_type") {
        payload.insert(
            "firing_filter_type".to_string(),
            serde_json::json!(firing_filter_type),
        );
    }

    if !discarded_cards.is_empty() {
        payload.insert(
            "discarded_cards".to_string(),
            serde_json::Value::Array(discarded_cards.iter().map(card_log_value).collect()),
        );
    }

    if let Some(blocked_filter) = blocked_filter {
        payload.insert("blocked_filter".to_string(), card_log_value(blocked_filter));
    }

    payload.insert(
        "current_player_health_before".to_string(),
        serde_json::json!(current_player_health_before),
    );
    payload.insert("stays_active".to_string(), serde_json::json!(stays_active));

    match card.id.as_str() {
        "repair" => {
            payload.insert("healed".to_string(), serde_json::json!(1));
        }
        "repair-kit" => {
            payload.insert("healed".to_string(), serde_json::json!(3));
        }
        "draw-2" => {
            payload.insert("cards_drawn".to_string(), serde_json::json!(2));
        }
        "lottery" => {
            payload.insert("cards_drawn".to_string(), serde_json::json!(5));
        }
        "painful-draw" => {
            payload.insert("cards_drawn".to_string(), serde_json::json!(3));
            payload.insert("self_damage".to_string(), serde_json::json!(2));
        }
        "more-ammo" => {
            payload.insert("more_ammo_added".to_string(), serde_json::json!(1));
        }
        "spray" => {
            payload.insert("self_healed".to_string(), serde_json::json!(1));
            payload.insert("other_players_damage".to_string(), serde_json::json!(3));
        }
        "health-hazard" => {
            payload.insert(
                "target_health_randomized".to_string(),
                serde_json::json!(true),
            );
            if let Some(target_health_before) = target_health_before {
                payload.insert(
                    "target_health_before".to_string(),
                    serde_json::json!(target_health_before),
                );
            }
        }
        "nuke" => {
            payload.insert("target_health_set_to".to_string(), serde_json::json!(2));
        }
        "crack" => {
            payload.insert("target_damage".to_string(), serde_json::json!(2));
        }
        "dent" => {
            payload.insert("target_damage".to_string(), serde_json::json!(1));
        }
        "stolen-parts" => {
            payload.insert("target_damage".to_string(), serde_json::json!(2));
            payload.insert("self_healed".to_string(), serde_json::json!(1));
        }
        "big-bomb" => {
            payload.insert("target_damage".to_string(), serde_json::json!(6));
        }
        "small-bomb" => {
            payload.insert("target_damage".to_string(), serde_json::json!(3));
        }
        "firing-filter" => {
            if let Some(firing_filter_type) = header_value(req, "firing_filter_type") {
                payload.insert(
                    "filter_type".to_string(),
                    serde_json::json!(firing_filter_type),
                );
            }
        }
        _ => {}
    }

    if card.id == "steal" {
        if let (Some(target_hand_before), Some(target_hand_after)) =
            (target_hand_before, target_hand_after)
        {
            if let Some(stolen_card) = removed_card(target_hand_before, target_hand_after) {
                payload.insert("stolen_card".to_string(), card_log_value(&stolen_card));
            }
        }
    }

    let message = if status == "played" {
        let discarded_cards_for_message: Vec<Card> = discarded_cards
            .iter()
            .filter(|discarded_card| {
                discarded_card.id != card.id
                    || discarded_card.name != card.name
                    || discarded_card.card_type != card.card_type
            })
            .cloned()
            .collect();

        format_play_event_message(
            username,
            card,
            target_name.as_deref(),
            header_value(req, "firing_filter_type").as_deref(),
            &discarded_cards_for_message,
            current_player_health_before,
            current_player_health_after,
        )
    } else {
        serde_json::Value::Object(payload).to_string()
    };

    game.push_server_chat_message(message);
}

fn blocking_firing_filter(
    game: &GameState,
    card: &Card,
    req: &Request<Body>,
) -> Option<(String, String)> {
    let Some(filter_type) = card.card_type.shooting_type() else {
        return None;
    };

    let target = if card.id == "distractor-missile" {
        let queued_card = header_value(req, "queuedcard")?;
        let queued_card_id = queued_card
            .split_once(':')
            .map(|(_, card_id)| card_id)
            .unwrap_or(queued_card.as_str());
        let selected_shooting = queued_card
            .split_once(':')
            .and_then(|(index, _)| index.parse::<usize>().ok())
            .and_then(|index| game.active_cards.active_calculated_shootings.get(index))
            .filter(|shooting| shooting.card_played.id == queued_card_id)
            .or_else(|| {
                game.active_cards
                    .active_calculated_shootings
                    .iter()
                    .find(|shooting| shooting.card_played.id == queued_card_id)
            })?;

        selected_shooting.owner.clone()
    } else {
        header_value(req, "target")?
    };

    let Some(active_filter) = game
        .active_cards
        .active_firing_filters
        .iter()
        .find(|filter| filter.owner == target && filter.filter_type == filter_type)
    else {
        return None;
    };

    Some((
        active_filter.owner.clone(),
        active_filter.card_played.name.clone(),
    ))
}

fn calculated_activation_targets(
    active: &ActiveCalculatedShooting,
    req: &Request<Body>,
) -> Vec<String> {
    if active.card_played.id == "decision-missile"
        && header_value(req, "decision_action").as_deref() == Some("draw")
    {
        return Vec::new();
    }

    if active.card_played.id == "multi-strike" {
        let Some(raw_allocations) = header_value(req, "multistrike_allocations") else {
            return Vec::new();
        };
        let Ok(allocations) = serde_json::from_str::<Vec<serde_json::Value>>(&raw_allocations)
        else {
            return Vec::new();
        };

        return allocations
            .into_iter()
            .filter(|allocation| {
                allocation
                    .get("damage")
                    .and_then(|damage| damage.as_i64())
                    .is_some_and(|damage| damage > 0)
            })
            .filter_map(|allocation| {
                allocation
                    .get("target")
                    .and_then(|target| target.as_str())
                    .map(str::to_string)
            })
            .collect();
    }

    header_value(req, "target").into_iter().collect()
}

fn blocking_calculated_activation_filter(
    game: &GameState,
    active: &ActiveCalculatedShooting,
    req: &Request<Body>,
) -> Option<(String, String)> {
    calculated_activation_targets(active, req)
        .into_iter()
        .find_map(|target| {
            game.active_cards
                .active_firing_filters
                .iter()
                .find(|filter| {
                    filter.owner == target && filter.filter_type == ShootingType::Calculated
                })
                .map(|filter| (filter.owner.clone(), filter.card_played.name.clone()))
        })
}

fn play_card_response(req: &Request<Body>, state: &Arc<ServerState>) -> Response<Body> {
    let joincode = header_value(req, "joincode");
    let username = header_value(req, "username");
    let cardid = header_value(req, "cardid");

    let (Some(joincode), Some(username), Some(cardid)) = (joincode, username, cardid) else {
        return text_response(
            StatusCode::BAD_REQUEST,
            "missing joincode, username, or cardid",
        );
    };

    let mut games = lock_or_recover(&state.games, "games");
    let Some(game) = games.get_mut(&joincode) else {
        return text_response(StatusCode::NOT_FOUND, "game not found for joincode");
    };

    if game.players.is_empty() {
        return text_response(StatusCode::BAD_REQUEST, "game has no players");
    }

    let player_index = game.current_turn_player;
    if game.players[player_index].name != username {
        return text_response(StatusCode::FORBIDDEN, "not your turn");
    }

    let hand = &game.players[player_index].hand;
    let card_index = hand.iter().position(|card| card.id == cardid).or_else(|| {
        cardid
            .parse::<usize>()
            .ok()
            .filter(|index| *index < hand.len())
    });

    let Some(card_index) = card_index else {
        return text_response(StatusCode::BAD_REQUEST, "card not found in hand");
    };

    let hand_before = game.players[player_index].hand.clone();
    let current_player_health_before = game.players[player_index].health;
    let target_name = header_value(req, "target");
    let (target_health_before, target_hand_before) = target_name
        .as_ref()
        .and_then(|name| {
            game.player_by_name(name)
                .map(|player| (Some(player.health), Some(player.hand.clone())))
        })
        .unwrap_or((None, None));

    let card = game.players[player_index].hand[card_index].clone();
    if !(card.can_be_played)(&card, game, req) {
        if let Some((filter_owner, blocking_filter_name)) = blocking_firing_filter(game, &card, req)
        {
            game.push_server_chat_message(format!(
                "{}'s {} blocked {}'s {}",
                filter_owner, blocking_filter_name, username, card.name
            ));
            return json_response(
                StatusCode::OK,
                serde_json::to_string(&serde_json::json!({"status": "blocked"})).unwrap(),
            );
        }
        log_play_event(
            game,
            &joincode,
            &username,
            &card,
            req,
            &hand_before,
            "rejected",
            &[],
            None,
            false,
            current_player_health_before,
            current_player_health_before,
            target_name.clone(),
            target_health_before,
            target_hand_before.as_deref(),
            target_hand_before.as_deref(),
        );
        return text_response(StatusCode::BAD_REQUEST, "card cannot be played");
    }

    let discard_pile_len_before_play = game.discard_pile.len();
    game.players[player_index].hand.remove(card_index);
    (card.play)(&card, game, player_index, req);

    let mut discarded_cards: Vec<Card> = game.discard_pile[discard_pile_len_before_play..].to_vec();

    match card.card_type {
        CardType::Shooting(_) => {
            if card.id == "distractor-missile" {
                game.turn_state.distractor_missile_played += 1;
            } else {
                game.turn_state.shooting_card_played += 1;
            }
        }
        CardType::Event => {
            game.turn_state.event_card_played = true;
        }
        _ => {}
    }
    game.turn_state.total_cards_played += 1;
    if !card_stays_active(&card) {
        game.discard_pile.push(card.clone());
        discarded_cards.push(card.clone());
    }

    let current_player_health_after = game
        .player_by_name(&username)
        .map(|player| player.health)
        .unwrap_or(0);

    let target_hand_after = target_name
        .as_ref()
        .and_then(|name| game.player_by_name(name).map(|player| player.hand.clone()));

    log_play_event(
        game,
        &joincode,
        &username,
        &card,
        req,
        &hand_before,
        "played",
        &discarded_cards,
        None,
        card_stays_active(&card),
        current_player_health_before,
        current_player_health_after,
        target_name.clone(),
        target_health_before,
        target_hand_before.as_deref(),
        target_hand_after.as_deref(),
    );

    json_response(
        StatusCode::OK,
        serde_json::to_string(&serde_json::json!({"status": "played"})).unwrap(),
    )
}

fn activate_calculated_shooting_response(
    req: &Request<Body>,
    state: &Arc<ServerState>,
) -> Response<Body> {
    let joincode = header_value(req, "joincode");
    let username = header_value(req, "username");
    let cardid = header_value(req, "cardid");

    let (Some(joincode), Some(username), Some(cardid)) = (joincode, username, cardid) else {
        return text_response(
            StatusCode::BAD_REQUEST,
            "missing joincode, username, or cardid",
        );
    };

    let mut games = lock_or_recover(&state.games, "games");
    let Some(game) = games.get_mut(&joincode) else {
        return text_response(StatusCode::NOT_FOUND, "game not found for joincode");
    };

    let Some(active_index) = game
        .active_cards
        .active_calculated_shootings
        .iter()
        .position(|active| {
            active.owner == username
                && active.card_played.id == cardid
                && active.turns_remaining == 0
        })
    else {
        return text_response(StatusCode::BAD_REQUEST, "active calculated card not found");
    };

    let active = game
        .active_cards
        .active_calculated_shootings
        .remove(active_index);
    let activated_card_id = active.card_played.id.clone();
    if let Some((filter_owner, blocking_filter_name)) =
        blocking_calculated_activation_filter(game, &active, req)
    {
        game.push_server_chat_message(format!(
            "{}'s {} blocked {}'s {}",
            filter_owner, blocking_filter_name, username, active.card_played.name
        ));
        game.discard_pile.push(active.card_played);
        return json_response(
            StatusCode::OK,
            serde_json::to_string(&serde_json::json!({"status": "blocked"})).unwrap(),
        );
    }

    (active.when_done)(game, &active, req);
    if activated_card_id == "multi-strike" {
        game.turn_state.shooting_locked = true;
    }
    game.discard_pile.push(active.card_played);

    json_response(
        StatusCode::OK,
        serde_json::to_string(&serde_json::json!({"status": "activated"})).unwrap(),
    )
}

fn generate_joincode() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(6);
    for _ in 0..6 {
        let letter = (b'A' + rng.gen_range(0..26)) as char;
        code.push(letter);
    }
    code
}

fn unique_joincode_for_state(state: &ServerState) -> String {
    let lobbies = lock_or_recover(&state.lobbies, "lobbies");
    let games = lock_or_recover(&state.games, "games");
    loop {
        let code = generate_joincode();
        if !lobbies.contains_key(&code) && !games.contains_key(&code) {
            return code;
        }
    }
}

#[derive(Serialize)]
struct GameStateResponse<'a> {
    players: Vec<PlayerResponse<'a>>,
    id: &'a str,
    current_turn_player: usize,
    draw_pile_count: usize,
    discard_pile: &'a [Card],
    turn_state: &'a TurnState,
    active_cards: &'a ActiveCards,
    game_settings: &'a GameSettings,
    chat_messages: &'a [ChatMessage],
}

#[derive(Serialize)]
struct MyGameResponse {
    joincode: String,
    username: String,
    status: String,
    players: Vec<MyGamePlayerResponse>,
    current_turn_player: Option<String>,
}

#[derive(Serialize)]
struct MyGamePlayerResponse {
    name: String,
    picture: Option<String>,
}

#[derive(Serialize)]
struct ShooSettingsResponse {
    username: String,
    picture: String,
    has_custom_username: bool,
    has_custom_picture: bool,
}

#[derive(Clone, Debug)]
struct VerifiedShooIdentity {
    user_id: String,
    default_username: String,
    default_picture: String,
}

#[derive(Debug, Deserialize)]
struct ShooTokenClaims {
    pairwise_sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    picture: Option<String>,
}

#[derive(Serialize)]
struct PlayerResponse<'a> {
    name: &'a str,
    picture: Option<&'a str>,
    hand: &'a [Card],
    hand_count: usize,
    health: isize,
}

#[derive(Serialize)]
struct LobbyPlayerResponse<'a> {
    name: &'a str,
    picture: Option<&'a str>,
}

fn can_view_player_private_data(
    game: &GameState,
    player_name: &str,
    username: Option<&str>,
    shoo_user_id: Option<&str>,
) -> bool {
    if Some(player_name) != username {
        return false;
    }

    match game.shoo_identities.get(player_name) {
        Some(expected_shoo_user_id) => Some(expected_shoo_user_id.as_str()) == shoo_user_id,
        None => true,
    }
}

fn game_state_response_for_player<'a>(
    game: &'a GameState,
    username: Option<&str>,
    shoo_user_id: Option<&str>,
) -> GameStateResponse<'a> {
    let players = game
        .players
        .iter()
        .map(|player| PlayerResponse {
            name: &player.name,
            picture: game
                .shoo_pictures
                .get(&player.name)
                .map(|picture| picture.as_str()),
            hand: if can_view_player_private_data(game, &player.name, username, shoo_user_id) {
                &player.hand
            } else {
                &[]
            },
            hand_count: player.hand.len(),
            health: player.health,
        })
        .collect();

    GameStateResponse {
        players,
        id: &game.id,
        current_turn_player: game.current_turn_player,
        draw_pile_count: game.draw_pile.len(),
        discard_pile: &game.discard_pile,
        turn_state: &game.turn_state,
        active_cards: &game.active_cards,
        game_settings: &game.game_settings,
        chat_messages: &game.chat_messages,
    }
}

fn normalized_header_value(req: &Request<Body>, name: &str) -> String {
    header_value(req, name)
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

fn shoo_audience(settings: &ServerSettings) -> String {
    format!("origin:{}", server_connect_url(settings))
}

async fn verify_shoo_identity(
    req: &Request<Body>,
    settings: &ServerSettings,
) -> Result<Option<VerifiedShooIdentity>, String> {
    let Some(token) = header_value(req, "shoo_token").filter(|token| !token.trim().is_empty()) else {
        return Ok(None);
    };

    let header = decode_header(&token).map_err(|err| format!("invalid shoo token: {}", err))?;
    if header.alg != Algorithm::ES256 {
        return Err("invalid shoo token algorithm".to_string());
    }

    let Some(kid) = header.kid else {
        return Err("shoo token missing key id".to_string());
    };

    let jwks: JwkSet = reqwest::get(format!("{}/.well-known/jwks.json", SHOO_BASE_URL))
        .await
        .map_err(|err| format!("failed to fetch Shoo keys: {}", err))?
        .error_for_status()
        .map_err(|err| format!("failed to fetch Shoo keys: {}", err))?
        .json()
        .await
        .map_err(|err| format!("failed to parse Shoo keys: {}", err))?;

    let jwk = jwks
        .find(&kid)
        .ok_or_else(|| "shoo token key not found".to_string())?;
    let decoding_key =
        DecodingKey::from_jwk(jwk).map_err(|err| format!("invalid Shoo key: {}", err))?;

    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_issuer(&[SHOO_ISSUER]);
    validation.set_audience(&[shoo_audience(settings)]);

    let token = decode::<ShooTokenClaims>(&token, &decoding_key, &validation)
        .map_err(|err| format!("invalid shoo token: {}", err))?;
    let claims = token.claims;
    if claims.pairwise_sub.trim().is_empty() {
        return Err("shoo token missing pairwise_sub".to_string());
    }

    Ok(Some(VerifiedShooIdentity {
        user_id: claims.pairwise_sub,
        default_username: claims
            .name
            .or(claims.email)
            .unwrap_or_else(|| "".to_string()),
        default_picture: claims.picture.unwrap_or_default(),
    }))
}

fn shoo_bad_request(error: String) -> Response<Body> {
    text_response(StatusCode::UNAUTHORIZED, error)
}

fn shoo_settings_response(
    state: &Arc<ServerState>,
    shoo_user_id: &str,
    default_username: String,
    default_picture: String,
) -> ShooSettingsResponse {
    let settings = lock_or_recover(&state.shoo_settings, "shoo_settings");
    let saved = settings.get(shoo_user_id);
    ShooSettingsResponse {
        username: saved
            .and_then(|settings| settings.username.clone())
            .unwrap_or(default_username),
        picture: saved
            .and_then(|settings| settings.picture.clone())
            .unwrap_or(default_picture),
        has_custom_username: saved
            .and_then(|settings| settings.username.as_ref())
            .is_some(),
        has_custom_picture: saved
            .and_then(|settings| settings.picture.as_ref())
            .is_some(),
    }
}

fn load_certs(path: &str) -> Vec<Certificate> {
    let certfile = File::open(path).expect("cannot open certificate");
    let mut reader = BufReader::new(certfile);
    certs(&mut reader)
        .expect("failed to read certificates")
        .into_iter()
        .map(Certificate)
        .collect()
}

fn load_private_key(path: &str) -> PrivateKey {
    let keyfile = File::open(path).expect("cannot open private key");
    let mut reader = BufReader::new(keyfile);

    // Try pkcs8 first
    let pkcs8 = pkcs8_private_keys(&mut reader).expect("cannot parse pkcs8 private key");
    if !pkcs8.is_empty() {
        return PrivateKey(pkcs8[0].clone());
    }

    // Re-open and try rsa keys
    let keyfile = File::open(path).expect("cannot open private key");
    let mut reader = BufReader::new(keyfile);
    let rsa = rsa_private_keys(&mut reader).expect("cannot parse rsa private key");
    if !rsa.is_empty() {
        return PrivateKey(rsa[0].clone());
    }

    panic!("no private keys found in {}", path);
}

async fn handle_request(
    req: Request<Body>,
    state: Arc<ServerState>,
    server_settings: ServerSettings,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path().to_string();
    let shoo_identity = match verify_shoo_identity(&req, &server_settings).await {
        Ok(identity) => identity,
        Err(error) => return Ok(shoo_bad_request(error)),
    };
    if shoo_identity.is_none()
        && (req.headers().contains_key("shoo_user_id")
            || req.headers().contains_key("shoo_picture")
            || req.headers().contains_key("shoo_default_username")
            || req.headers().contains_key("shoo_default_picture"))
    {
        return Ok(shoo_bad_request("missing shoo_token".to_string()));
    }

    match (req.method(), path.as_str()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") | (&Method::GET, "/shoo/callback") => {
            let p = format!("{}/index.html", WEBSITE_DIR);
            serve_file(&p, "text/html; charset=utf-8").await
        }
        (&Method::GET, "/css.css") => {
            let p = format!("{}/css.css", WEBSITE_DIR);
            serve_file(&p, "text/css; charset=utf-8").await
        }
        (&Method::GET, image_path) if image_path.starts_with("/images/") => {
            let p = format!("{}/{}", IMAGES_DIR, &image_path[8..]);
            serve_image(&p).await
        }
        (&Method::GET, "/gameScreen.js")
        | (&Method::GET, "/networkFunctions.js")
        | (&Method::GET, "/playCard.js")
        | (&Method::GET, "/chat.js")
        | (&Method::GET, "/auth.js") => {
            let p = format!("{}/{}", WEBSITE_DIR, &path[1..]);
            serve_file(&p, "application/javascript").await
        }
        (&Method::GET, "/lobby.js") => serve_lobby_js(&server_settings).await,
        (&Method::GET, "/checkOnline") => Ok(text_response(StatusCode::OK, "Online")),
        (&Method::GET, "/shooSettings") => {
            match shoo_identity.as_ref() {
                Some(shoo_identity) => {
                    Ok(json_response(
                        StatusCode::OK,
                        serde_json::to_string(&shoo_settings_response(
                            &state,
                            &shoo_identity.user_id,
                            shoo_identity.default_username.clone(),
                            shoo_identity.default_picture.clone(),
                        ))
                        .unwrap(),
                    ))
                }
                None => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing shoo_token",
                )),
            }
        }
        (&Method::POST, "/shooSettings") => {
            match shoo_identity.as_ref() {
                Some(shoo_identity) => {
                    let default_username = shoo_identity.default_username.clone();
                    let default_picture = shoo_identity.default_picture.clone();
                    let username = normalized_header_value(&req, "settings_username");
                    let picture = normalized_header_value(&req, "settings_picture");

                    let custom_username = if username.is_empty() || username == default_username {
                        None
                    } else {
                        Some(username)
                    };
                    let custom_picture = if picture.is_empty() || picture == default_picture {
                        None
                    } else {
                        Some(picture)
                    };

                    {
                        let mut settings = lock_or_recover(&state.shoo_settings, "shoo_settings");
                        if custom_username.is_none() && custom_picture.is_none() {
                            settings.remove(&shoo_identity.user_id);
                        } else {
                            settings.insert(
                                shoo_identity.user_id.clone(),
                                ShooUserSettings {
                                    username: custom_username,
                                    picture: custom_picture,
                                },
                            );
                        }
                    }

                    Ok(json_response(
                        StatusCode::OK,
                        serde_json::to_string(&shoo_settings_response(
                            &state,
                            &shoo_identity.user_id,
                            default_username,
                            default_picture,
                        ))
                        .unwrap(),
                    ))
                }
                None => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing shoo_token",
                )),
            }
        }
        (&Method::POST, "/createGame") => {
            let username = header_value(&req, "username");
            let shoo_user_id = shoo_identity.as_ref().map(|identity| identity.user_id.clone());
            let mut shoo_picture = shoo_identity
                .as_ref()
                .map(|identity| identity.default_picture.clone())
                .unwrap_or_default();
            if let Some(shoo_user_id) = shoo_user_id.as_ref() {
                if let Some(custom_picture) = lock_or_recover(&state.shoo_settings, "shoo_settings")
                    .get(shoo_user_id)
                    .and_then(|settings| settings.picture.clone())
                {
                    shoo_picture = custom_picture;
                }
            }
            match username {
                Some(username) => {
                    let joincode = unique_joincode_for_state(&state);
                    let mut lobbies = lock_or_recover(&state.lobbies, "lobbies");
                    lobbies.insert(
                        joincode.clone(),
                        LobbyState::new(username, shoo_user_id, shoo_picture),
                    );
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/plain; charset=utf-8")
                        .header("access-control-allow-origin", "*")
                        .header("access-control-expose-headers", "joincode")
                        .header("joincode", joincode.clone())
                        .body(Body::from(joincode))
                        .unwrap())
                }
                _ => Ok(text_response(StatusCode::BAD_REQUEST, "missing username")),
            }
        }
        (&Method::POST, "/joinGame") => {
            let username = header_value(&req, "username");
            let joincode = header_value(&req, "joincode");
            let shoo_user_id = shoo_identity.as_ref().map(|identity| identity.user_id.clone());
            let mut shoo_picture = shoo_identity
                .as_ref()
                .map(|identity| identity.default_picture.clone())
                .unwrap_or_default();
            if let Some(shoo_user_id) = shoo_user_id.as_ref() {
                if let Some(custom_picture) = lock_or_recover(&state.shoo_settings, "shoo_settings")
                    .get(shoo_user_id)
                    .and_then(|settings| settings.picture.clone())
                {
                    shoo_picture = custom_picture;
                }
            }
            match (username, joincode) {
                (Some(username), Some(joincode)) => {
                    let mut lobbies = lock_or_recover(&state.lobbies, "lobbies");
                    match lobbies.get_mut(&joincode) {
                        Some(lobby) => {
                            if lobby.players.iter().any(|player| player == &username) {
                                Ok(text_response(
                                    StatusCode::OK,
                                    "no duplicate usernames allowed",
                                ))
                            } else {
                                lobby.players.push(username.clone());
                                if let Some(shoo_user_id) = shoo_user_id {
                                    lobby.shoo_identities.insert(username.clone(), shoo_user_id);
                                    if !shoo_picture.is_empty() {
                                        lobby.shoo_pictures.insert(username, shoo_picture);
                                    }
                                }
                                Ok(text_response(StatusCode::OK, "joined game lobby"))
                            }
                        }
                        None => Ok(text_response(StatusCode::OK, "game does not exist")),
                    }
                }
                _ => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing username or joincode",
                )),
            }
        }
        (&Method::GET, "/myGames") => {
            match shoo_identity.as_ref() {
                Some(shoo_identity) => {
                    let shoo_user_id = &shoo_identity.user_id;
                    let mut games_for_user = Vec::new();

                    {
                        let lobbies = lock_or_recover(&state.lobbies, "lobbies");
                        for (joincode, lobby) in lobbies.iter() {
                            for (username, identity) in lobby.shoo_identities.iter() {
                                if identity == shoo_user_id {
                                    games_for_user.push(MyGameResponse {
                                        joincode: joincode.clone(),
                                        username: username.clone(),
                                        status: "lobby".to_string(),
                                        players: lobby
                                            .players
                                            .iter()
                                            .map(|player| MyGamePlayerResponse {
                                                name: player.clone(),
                                                picture: lobby.shoo_pictures.get(player).cloned(),
                                            })
                                            .collect(),
                                        current_turn_player: None,
                                    });
                                }
                            }
                        }
                    }

                    {
                        let games = lock_or_recover(&state.games, "games");
                        for (joincode, game) in games.iter() {
                            for (username, identity) in game.shoo_identities.iter() {
                                if identity == shoo_user_id {
                                    games_for_user.push(MyGameResponse {
                                        joincode: joincode.clone(),
                                        username: username.clone(),
                                        status: "started".to_string(),
                                        players: game
                                            .players
                                            .iter()
                                            .map(|player| MyGamePlayerResponse {
                                                name: player.name.clone(),
                                                picture: game
                                                    .shoo_pictures
                                                    .get(&player.name)
                                                    .cloned(),
                                            })
                                            .collect(),
                                        current_turn_player: game
                                            .players
                                            .get(game.current_turn_player)
                                            .map(|player| player.name.clone()),
                                    });
                                }
                            }
                        }
                    }

                    Ok(json_response(
                        StatusCode::OK,
                        serde_json::to_string(&games_for_user).unwrap(),
                    ))
                }
                None => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing shoo_token",
                )),
            }
        }
        (&Method::POST, "/leaveLobby") => {
            let username = header_value(&req, "username");
            let joincode = header_value(&req, "joincode");
            match (username, joincode) {
                (Some(username), Some(joincode)) => {
                    let mut lobbies = lock_or_recover(&state.lobbies, "lobbies");
                    match lobbies.get_mut(&joincode) {
                        Some(lobby) => {
                            if let Some(index) =
                                lobby.players.iter().position(|player| player == &username)
                            {
                                lobby.players.remove(index);
                                lobby.shoo_identities.remove(&username);
                                lobby.shoo_pictures.remove(&username);
                                if lobby.players.is_empty() {
                                    lobbies.remove(&joincode);
                                }
                                Ok(text_response(StatusCode::OK, "left game lobby"))
                            } else {
                                Ok(text_response(StatusCode::OK, "not in lobby"))
                            }
                        }
                        None => Ok(text_response(StatusCode::OK, "game does not exist")),
                    }
                }
                _ => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing username or joincode",
                )),
            }
        }
        (&Method::GET, "/lobbyState") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let lobbies = lock_or_recover(&state.lobbies, "lobbies");
                    match lobbies.get(&joincode) {
                        Some(lobby) => {
                            let host = lobby.players.first().cloned().unwrap_or_default();
                            let players: Vec<LobbyPlayerResponse> = lobby
                                .players
                                .iter()
                                .map(|player| LobbyPlayerResponse {
                                    name: player,
                                    picture: lobby
                                        .shoo_pictures
                                        .get(player)
                                        .map(|picture| picture.as_str()),
                                })
                                .collect();
                            Ok(Response::builder()
                                .status(StatusCode::OK)
                                .header("content-type", "application/json")
                                .header("access-control-allow-origin", "*")
                                .header("access-control-expose-headers", "host-username")
                                .header("host-username", host)
                                .body(Body::from(serde_json::to_string(&players).unwrap()))
                                .unwrap())
                        }
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "lobby not found for the given joincode",
                        )),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::POST, "/startGame") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let mut lobbies = lock_or_recover(&state.lobbies, "lobbies");
                    match lobbies.get(&joincode) {
                        Some(lobby) if lobby.players.len() < 2 => Ok(text_response(
                            StatusCode::BAD_REQUEST,
                            "game requires at least 2 players",
                        )),
                        Some(_) => {
                            let lobby = lobbies.remove(&joincode).unwrap();
                            let default_game_settings = "{\"starting_hand_size\":4,\"starting_health\":10,\"max_health\":10,\"play_heal_on_others\":false,\"revive_others_with_heal\":false}";
                            let settings_str = header_value(&req, "settings")
                                .unwrap_or(default_game_settings.to_string());
                            let settings: GameSettings =
                                serde_json::from_str(&settings_str).unwrap();
                            let mut game_state = GameState::new(
                                joincode.clone(),
                                lobby.players.clone(),
                                lobby.shoo_identities,
                                lobby.shoo_pictures,
                                settings,
                            );
                            game_state.chat_messages = lobby.chat_messages;
                            game_state.start_game();
                            lock_or_recover(&state.games, "games")
                                .insert(joincode.clone(), game_state);
                            lock_or_recover(&state.started_games, "started_games")
                                .insert(joincode.clone());
                            Ok(text_response(StatusCode::OK, "Game Started"))
                        }
                        None => Ok(text_response(StatusCode::NOT_FOUND, "game does not exist")),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::GET, "/checkGameStarted") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let started =
                        lock_or_recover(&state.started_games, "started_games").contains(&joincode);
                    Ok(text_response(
                        StatusCode::OK,
                        if started { "yes" } else { "no" },
                    ))
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::POST, "/endTurn") => {
            let joincode: Option<String> = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let mut games = lock_or_recover(&state.games, "games");
                    match games.get_mut(&joincode) {
                        Some(game) => {
                            game.next_turn();
                            let body = serde_json::to_string(&serde_json::json!({"current_turn_player": game.current_turn_player})).unwrap();
                            Ok(json_response(StatusCode::OK, body))
                        }
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "game not found for joincode",
                        )),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::GET, "/gameState") => {
            let joincode: Option<String> = header_value(&req, "joincode");
            let username: Option<String> = header_value(&req, "username");
            let shoo_user_id = shoo_identity.as_ref().map(|identity| identity.user_id.as_str());
            match joincode {
                Some(joincode) => {
                    let mut games = lock_or_recover(&state.games, "games");
                    match games.get_mut(&joincode) {
                        Some(game) => {
                            let body = serde_json::to_string(&game_state_response_for_player(
                                game,
                                username.as_deref(),
                                shoo_user_id,
                            ))
                            .unwrap();
                            Ok(json_response(StatusCode::OK, body))
                        }
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "game not found for joincode",
                        )),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::POST, "/sendChat") => {
            let joincode = header_value(&req, "joincode");
            let sender = header_value(&req, "username").unwrap_or_else(|| "Anonymous".to_string());
            let timestamp = current_timestamp();
            let message = header_value(&req, "text");

            match (joincode, message) {
                (Some(joincode), Some(message)) => {
                    let chat_message = ChatMessage {
                        sender,
                        message,
                        timestamp,
                    };

                    {
                        let mut games = lock_or_recover(&state.games, "games");
                        if let Some(game) = games.get_mut(&joincode) {
                            game.chat_messages.push(chat_message);
                            return Ok(text_response(StatusCode::OK, "sent"));
                        }
                    }

                    {
                        let mut lobbies = lock_or_recover(&state.lobbies, "lobbies");
                        if let Some(lobby) = lobbies.get_mut(&joincode) {
                            lobby.chat_messages.push(chat_message);
                            return Ok(text_response(StatusCode::OK, "sent"));
                        }
                    }

                    Ok(text_response(
                        StatusCode::NOT_FOUND,
                        "lobby/game not found for joincode",
                    ))
                }
                (None, _) => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
                (_, None) => Ok(text_response(StatusCode::BAD_REQUEST, "missing text")),
            }
        }
        (&Method::GET, "/getChat") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    {
                        let games = lock_or_recover(&state.games, "games");
                        if let Some(game) = games.get(&joincode) {
                            let body = serde_json::to_string(&game.chat_messages).unwrap();
                            return Ok(json_response(StatusCode::OK, body));
                        }
                    }

                    {
                        let lobbies = lock_or_recover(&state.lobbies, "lobbies");
                        if let Some(lobby) = lobbies.get(&joincode) {
                            let body = serde_json::to_string(&lobby.chat_messages).unwrap();
                            return Ok(json_response(StatusCode::OK, body));
                        }
                    }

                    Ok(text_response(
                        StatusCode::NOT_FOUND,
                        "lobby/game not found for joincode",
                    ))
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::GET, "/getDeck") => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&all_cards()).unwrap()))
            .unwrap()),
        (&Method::POST, "/quitGame") => {
            let joincode = header_value(&req, "joincode");
            let username = header_value(&req, "username");
            match (joincode, username) {
                (Some(joincode), Some(username)) => {
                    {
                        let mut games = lock_or_recover(&state.games, "games");
                        if let Some(game) = games.get_mut(&joincode) {
                            match game.players.iter().position(|p| p.name == username) {
                                Some(player_index) => {
                                    game.push_server_chat_message(format!("{} quit", username));
                                    if game.remove_player(player_index, true) {
                                        games.remove(&joincode);
                                    } else {
                                        game.shoo_identities.remove(&username);
                                        game.shoo_pictures.remove(&username);
                                    }

                                    return Ok(text_response(StatusCode::OK, "quit"));
                                }
                                None => {
                                    return Ok(text_response(
                                        StatusCode::NOT_FOUND,
                                        "player not found in game",
                                    ));
                                }
                            }
                        }
                    }

                    let mut lobbies = lock_or_recover(&state.lobbies, "lobbies");
                    match lobbies.get_mut(&joincode) {
                        Some(lobby) => match lobby.players.iter().position(|p| p == &username) {
                            Some(player_index) => {
                                lobby.players.remove(player_index);
                                if lobby.players.is_empty() {
                                    lobbies.remove(&joincode);
                                } else {
                                    lobby.shoo_identities.remove(&username);
                                    lobby.shoo_pictures.remove(&username);
                                }

                                Ok(text_response(StatusCode::OK, "quit"))
                            }
                            None => Ok(text_response(
                                StatusCode::NOT_FOUND,
                                "player not found in game",
                            )),
                        },
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "game not found for joincode",
                        )),
                    }
                }
                (None, _) => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
                (_, None) => Ok(text_response(StatusCode::BAD_REQUEST, "missing username")),
            }
        }
        (&Method::POST, "/playCard") | (&Method::POST, "/playcard") => {
            Ok(play_card_response(&req, &state))
        }
        (&Method::POST, "/activateCalculatedShooting") => {
            Ok(activate_calculated_shooting_response(&req, &state))
        }
        (&Method::POST, "/activateDelayedCard") | (&Method::POST, "/useRadar") => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load certificates from the directory containing the running binary
    let exe_path = std::env::current_exe().expect("cannot determine current exe path");
    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // Determine base directory for certificates. If the binary appears to be
    // inside a `target` directory (typical when running via `cargo run`),
    // look two folders up (repo root). Otherwise use the binary directory.
    let mut cert_base = exe_dir.clone();
    if exe_dir.iter().any(|comp| comp == "target") {
        if let Some(two_up) = exe_dir.parent().and_then(|p| p.parent()) {
            cert_base = two_up.to_path_buf();
            eprintln!(
                "Running from cargo; looking for certs at {}",
                cert_base.display()
            );
        }
    }

    let cert_path = cert_base.join("certificate.crt");
    let key_path = cert_base.join("private.key");
    let ca_path = cert_base.join("ca_bundle.crt");

    // Require cert and key to exist at the chosen location; fail otherwise.
    if !cert_path.exists() || !key_path.exists() {
        eprintln!(
            "Certificate or key not found in {}.\nPlace certificate.crt and private.key in that directory.",
            cert_base.display()
        );
        return Err("certificate files not found".into());
    }

    // Load leaf cert(s)
    let mut certs = load_certs(
        cert_path
            .to_str()
            .expect("certificate path is not valid UTF-8"),
    );
    // If a CA bundle exists, append its certs to form a chain
    if ca_path.exists() {
        let mut ca_certs = load_certs(ca_path.to_str().expect("ca bundle path is not valid UTF-8"));
        certs.append(&mut ca_certs);
    }

    let key = load_private_key(
        key_path
            .to_str()
            .expect("private key path is not valid UTF-8"),
    );

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad certificates/private key");

    let acceptor = TlsAcceptor::from(Arc::new(config));

    let server_settings_path = "server_settings.json";
    let server_settings = load_server_settings(server_settings_path);

    let state_file_path = "server_state.json";
    let state = if std::path::Path::new(state_file_path).exists() {
        println!("Loading state from {}", state_file_path);
        let data =
            std::fs::read_to_string(state_file_path).expect("Failed to read server_state.json");
        let server_state: ServerState =
            serde_json::from_str(&data).expect("Failed to parse server_state.json");
        server_state.hydrate();
        Arc::new(server_state)
    } else {
        Arc::new(ServerState::new())
    };

    let state_for_auto_save = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            if let Ok(json) = serde_json::to_string_pretty(&*state_for_auto_save) {
                if let Err(e) = tokio::fs::write(state_file_path, json).await {
                    eprintln!("Failed to auto-save state: {}", e);
                }
            }
        }
    });

    let state_for_shutdown = state.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        println!("Received Ctrl-C, saving state...");
        if let Ok(json) = serde_json::to_string_pretty(&*state_for_shutdown) {
            if let Err(e) = std::fs::write(state_file_path, json) {
                eprintln!("Failed to save state on shutdown: {}", e);
            } else {
                println!("State saved successfully.");
            }
        }
        std::process::exit(0);
    });

    let bind_ip: std::net::IpAddr = server_settings
        .bind_ip
        .parse()
        .expect("bind_ip in server_settings.json must be a valid IP address");
    let addr = SocketAddr::new(bind_ip, server_settings.port);
    let listener = TcpListener::bind(addr).await?;

    println!(
        "HTTPS server listening on {}",
        server_connect_url(&server_settings)
    );

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let state = state.clone();
        let server_settings = server_settings.clone();

        tokio::spawn(async move {
            let peer = peer_addr;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    let service = service_fn(move |req| {
                        handle_request(req, state.clone(), server_settings.clone())
                    });
                    if let Err(err) = hyper::server::conn::Http::new()
                        .serve_connection(tls_stream, service)
                        .await
                    {
                        eprintln!("Error serving connection from {}: {}", peer, err);
                    }
                }
                Err(err) => {
                    eprintln!("TLS handshake failed for {}: {}", peer, err);
                }
            }
        });
    }
}
