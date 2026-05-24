use crate::{header_value, structs::*};
use hyper::{Body, Request};
use rand::{Rng, thread_rng};

pub fn stub_play(_: &Card, _: &mut GameState, _: usize, _: &Request<Body>) {}

pub fn can_never_play(_: &Card, _: &GameState, _: &Request<Body>) -> bool {
    return false;
}

pub fn can_always_play(_: &Card, _: &GameState, _: &Request<Body>) -> bool {
    return true;
}

pub fn stub_when_done() {}

pub fn all_cards() -> Vec<Card> {
    let card_templates = vec![
        Card {
            name: "Repair".to_string(),
            id: "repair".to_string(),
            card_type: CardType::Support,
            can_be_played: can_play_repair,
            play: play_repair,
            count: 6,
        },
        Card {
            name: "Radar".to_string(),
            id: "radar".to_string(),
            card_type: CardType::Support,
            can_be_played: can_play_radar,
            play: stub_play,
            count: 2,
        },
        Card {
            name: "Repair Kit".to_string(),
            id: "repair-kit".to_string(),
            card_type: CardType::Support,
            can_be_played: can_play_repair,
            play: play_repair_kit,
            count: 3,
        },
        Card {
            name: "Armor".to_string(),
            id: "armor".to_string(),
            card_type: CardType::Support,
            can_be_played: can_never_play,
            play: stub_play,
            count: 5,
        },
        Card {
            name: "Cold War".to_string(),
            id: "cold-war".to_string(),
            card_type: CardType::Support,
            can_be_played: can_play_cold_war,
            play: stub_play,
            count: 3,
        },
        Card {
            name: "Last Stand".to_string(),
            id: "last-stand".to_string(),
            card_type: CardType::Support,
            can_be_played: can_never_play,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Firing Filter".to_string(),
            id: "firing-filter".to_string(),
            card_type: CardType::Support,
            can_be_played: can_play_firing_filter,
            play: stub_play,
            count: 2,
        },
        Card {
            name: "Steal".to_string(),
            id: "steal".to_string(),
            card_type: CardType::Plus,
            can_be_played: can_play_steal,
            play: play_steal,
            count: 6,
        },
        Card {
            name: "Draw 2".to_string(),
            id: "draw-2".to_string(),
            card_type: CardType::Plus,
            can_be_played: can_always_play,
            play: play_draw_2,
            count: 3,
        },
        Card {
            name: "Helpful Hand".to_string(),
            id: "helpful-hand".to_string(),
            card_type: CardType::Plus,
            can_be_played: can_play_helpful_hand,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Painful Draw".to_string(),
            id: "painful-draw".to_string(),
            card_type: CardType::Plus,
            can_be_played: can_always_play,
            play: play_painful_draw,
            count: 2,
        },
        Card {
            name: "Crack".to_string(),
            id: "crack".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_quick_shooting,
            play: play_crack,
            count: 5,
        },
        Card {
            name: "Dent".to_string(),
            id: "dent".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_quick_shooting,
            play: play_dent,
            count: 10,
        },
        Card {
            name: "Stolen Parts".to_string(),
            id: "stolen-parts".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_quick_shooting,
            play: play_stolen_parts,
            count: 5,
        },
        Card {
            name: "Distractor Missile".to_string(),
            id: "distractor-missile".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_distractor_missile,
            play: stub_play,
            count: 8,
        },
        Card {
            name: "Aimed Missile".to_string(),
            id: "aimed-missile".to_string(),
            card_type: CardType::Shooting(ShootingType::Calculated),
            can_be_played: can_play_calculated_shooting,
            play: stub_play,
            count: 10,
        },
        Card {
            name: "Locked On".to_string(),
            id: "locked-on".to_string(),
            card_type: CardType::Shooting(ShootingType::Calculated),
            can_be_played: can_play_calculated_shooting,
            play: stub_play,
            count: 4,
        },
        Card {
            name: "Multi Strike".to_string(),
            id: "multi-strike".to_string(),
            card_type: CardType::Shooting(ShootingType::Calculated),
            can_be_played: can_play_calculated_shooting,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Descision Missile".to_string(),
            id: "descision-missile".to_string(),
            card_type: CardType::Shooting(ShootingType::Calculated),
            can_be_played: can_play_calculated_shooting,
            play: stub_play,
            count: 3,
        },
        Card {
            name: "Big Bomb".to_string(),
            id: "big-bomb".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_boom_shooting,
            play: play_big_bomb,
            count: 4,
        },
        Card {
            name: "Small Bomb".to_string(),
            id: "small-bomb".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_boom_shooting,
            play: play_small_bomb,
            count: 10,
        },
        Card {
            name: "Landmine".to_string(),
            id: "landmine".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_boom_shooting,
            play: play_landmine,
            count: 1,
        },
        Card {
            name: "Nuke".to_string(),
            id: "nuke".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_nuke,
            play: play_nuke,
            count: 1,
        },
        Card {
            name: "No Shooting".to_string(),
            id: "no-shooting".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_no_shooting,
            play: play_no_shooting,
            count: 2,
        },
        Card {
            name: "More Ammo".to_string(),
            id: "more-ammo".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_more_ammo,
            play: play_more_ammo,
            count: 2,
        },
        Card {
            name: "New Model".to_string(),
            id: "new-model".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_new_model,
            play: stub_play,
            count: 2,
        },
        Card {
            name: "Airstrike".to_string(),
            id: "airstrike".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_airstrike,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Spray".to_string(),
            id: "spray".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_event,
            play: play_spray,
            count: 1,
        },
        Card {
            name: "Lottery".to_string(),
            id: "lottery".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_lottery,
            play: play_lottery,
            count: 1,
        },
        Card {
            name: "Health Hazard".to_string(),
            id: "health-hazard".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_event,
            play: play_health_hazard,
            count: 2,
        },
        Card {
            name: "Recycle".to_string(),
            id: "recycle".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_event,
            play: stub_play,
            count: 1,
        },
    ];

    let mut cards: Vec<Card> = Vec::new();
    for card_template in card_templates.into_iter() {
        for _ in 0..card_template.count {
            cards.push(card_template.clone());
        }
    }
    return cards;
}

fn parse_shooting_type(value: &str) -> Option<ShootingType> {
    match value {
        "Quick" => Some(ShootingType::Quick),
        "Calculated" => Some(ShootingType::Calculated),
        "Boom" => Some(ShootingType::Boom),
        _ => None,
    }
}

fn can_play_firing_filter(_: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let firing_filter_type_str = headers
        .get("firing_filter_type")
        .and_then(|value| value.to_str().ok());

    let Some(firing_filter_type_str) = firing_filter_type_str else {
        return false;
    };

    let Some(filter_type) = parse_shooting_type(firing_filter_type_str) else {
        return false;
    };

    // Check if the current player already has an active firing filter of this type
    let current_player_name = &game_state.players[game_state.current_turn_player].name;
    for active_filter in &game_state.active_cards.active_firing_filters {
        if active_filter.owner == *current_player_name && active_filter.filter_type == filter_type {
            return false;
        }
    }
    return true;
}

fn can_play_repair(_: &Card, game_state: &GameState, _: &Request<Body>) -> bool {
    if game_state.players[game_state.current_turn_player].health >= 10 {
        return false;
    }
    return true;
}

fn can_play_radar(_: &Card, _: &GameState, _: &Request<Body>) -> bool {
    return true;
}

fn can_play_cold_war(_: &Card, game_state: &GameState, _: &Request<Body>) -> bool {
    if game_state
        .active_cards
        .active_calculated_shootings
        .is_empty()
    {
        return false;
    }
    return true;
}

fn can_play_nuke(_card: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    if game_state
        .player_by_name(&header_value(req, "target").unwrap())
        .unwrap()
        .health
        == 2
    {
        return false;
    }
    if game_state.turn_state.total_cards_played != 0 {
        return false;
    }
    return can_play_boom_shooting(_card, game_state, req);
}

fn can_play_distractor_missile(_card: &Card, game_state: &GameState, _req: &Request<Body>) -> bool {
    if game_state
        .active_cards
        .active_calculated_shootings
        .is_empty()
    {
        return false;
    }
    return can_play_quick_shooting(_card, game_state, _req);
}

fn can_play_quick_shooting(_: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_name = headers.get("target").and_then(|value| value.to_str().ok());

    if let Some(target_name) = target_player_name {
        for active_filter in &game_state.active_cards.active_firing_filters {
            if active_filter.owner == target_name
                && active_filter.filter_type == ShootingType::Quick
            {
                return false;
            }
        }
    }
    return can_play_shooting(game_state);
}

fn can_play_calculated_shooting(_: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_name = headers.get("target").and_then(|value| value.to_str().ok());

    if let Some(target_name) = target_player_name {
        for active_filter in &game_state.active_cards.active_firing_filters {
            if active_filter.owner == target_name
                && active_filter.filter_type == ShootingType::Calculated
            {
                return false;
            }
        }
    }
    return can_play_shooting(game_state);
}

fn can_play_boom_shooting(_: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_name = headers.get("target").and_then(|value| value.to_str().ok());

    if let Some(target_name) = target_player_name {
        for active_filter in &game_state.active_cards.active_firing_filters {
            if active_filter.owner == target_name && active_filter.filter_type == ShootingType::Boom
            {
                return false;
            }
        }
    }
    return can_play_shooting(game_state);
}

fn can_play_shooting(game_state: &GameState) -> bool {
    if game_state.no_shooting_played_by != -1
        && game_state.current_turn_player as isize != game_state.no_shooting_played_by
    {
        return false;
    }
    if game_state.turn_state.shooting_card_played >= game_state.turn_state.more_ammo_played + 1 {
        return false;
    }
    return true;
}

fn can_play_airstrike(_card: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_name = headers.get("target").and_then(|value| value.to_str().ok());

    let Some(target_player_name) = target_player_name else {
        return false;
    };

    let Some(target_player) = game_state.player_by_name(target_player_name) else {
        return false;
    };

    if target_player.hand.is_empty() {
        return false;
    }
    return can_play_event(_card, game_state, req);
}

fn can_play_more_ammo(_: &Card, game_state: &GameState, _: &Request<Body>) -> bool {
    if game_state.turn_state.event_card_played && game_state.turn_state.more_ammo_played == 0 {
        return false;
    }
    return true;
}

fn can_play_event(_: &Card, game_state: &GameState, _: &Request<Body>) -> bool {
    if game_state.turn_state.event_card_played {
        return false;
    }
    return true;
}

fn can_play_lottery(_: &Card, game_state: &GameState, _: &Request<Body>) -> bool {
    if game_state.turn_state.total_cards_played != 0 {
        return false;
    }
    return true;
}

fn can_play_new_model(_card: &Card, game_state: &GameState, _req: &Request<Body>) -> bool {
    let current_player = &game_state.players[game_state.current_turn_player];

    let shooting_cards_count = current_player
        .hand
        .iter()
        .filter(|card| card.card_type.is_shooting())
        .count();

    if shooting_cards_count < 2 {
        return false;
    }

    return can_play_event(_card, game_state, _req);
}

fn can_play_no_shooting(_card: &Card, game_state: &GameState, _req: &Request<Body>) -> bool {
    if game_state.turn_state.no_shooting_played {
        return false;
    }
    return can_play_event(_card, game_state, _req);
}

fn can_play_helpful_hand(_card: &Card, game_state: &GameState, _req: &Request<Body>) -> bool {
    if game_state.players[game_state.current_turn_player]
        .hand
        .len()
        > 1
    {
        return false;
    }
    return true;
}

fn can_play_steal(_card: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_name = headers.get("target").and_then(|value| value.to_str().ok());

    let Some(target_player_name) = target_player_name else {
        return false;
    };

    let Some(target_player) = game_state.player_by_name(target_player_name) else {
        return false;
    };

    if target_player.hand.is_empty() {
        return false;
    }
    return true;
}

fn play_repair(_: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.players[player_index].health += 1;
}
fn play_repair_kit(_: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.players[player_index].health += 3;
}
fn play_landmine(card: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.active_cards.landmine_played_by = player_index as isize;
    game_state.active_cards.landmine_card = Some(card.clone());
}
fn play_no_shooting(
    card: &Card,
    game_state: &mut GameState,
    player_index: usize,
    _: &Request<Body>,
) {
    game_state.no_shooting_played_by = player_index as isize;
    game_state.active_cards.no_shooting_card = Some(card.clone());
    game_state.turn_state.no_shooting_played = true;
}
fn play_steal(_: &Card, game_state: &mut GameState, player_index: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    let hand = &mut game_state.player_by_name_mut(&target).unwrap().hand;
    let card_index = thread_rng().gen_range(0..hand.len());
    let stolen_card = hand[card_index].clone();
    hand.remove(card_index);
    game_state.players[player_index].hand.push(stolen_card);
}
fn play_draw_2(_: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
}
fn play_lottery(_: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
}
fn play_painful_draw(_: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
    game_state.draw_card(player_index);
    game_state.damage_player(player_index, 2);
}
fn play_more_ammo(_: &Card, game_state: &mut GameState, _: usize, _: &Request<Body>) {
    game_state.turn_state.more_ammo_played += 1;
}
fn play_health_hazard(_: &Card, game_state: &mut GameState, _: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    game_state.player_by_name_mut(&target).unwrap().health = thread_rng().gen_range(1..=10);
}
fn play_spray(_: &Card, game_state: &mut GameState, player_index: usize, _: &Request<Body>) {
    game_state.players[player_index].health += 1;
    for i in 1..game_state.players.len() {
        game_state.damage_player(i, 3);
    }
}
fn play_nuke(_: &Card, game_state: &mut GameState, _: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    game_state.player_by_name_mut(&target).unwrap().health = 2;
}
fn play_big_bomb(_: &Card, game_state: &mut GameState, player_index: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    game_state.damage_player(game_state.player_index_from_name(&target).unwrap(), 6);
    game_state.damage_player(player_index, thread_rng().gen_bool(0.5) as isize * 5);
}
fn play_small_bomb(_: &Card, game_state: &mut GameState, player_index: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    game_state.damage_player(game_state.player_index_from_name(&target).unwrap(), 3);
    game_state.damage_player(player_index, thread_rng().gen_bool(0.5) as isize);
}
fn play_crack(_: &Card, game_state: &mut GameState, _: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    game_state.damage_player(game_state.player_index_from_name(&target).unwrap(), 2);
}
fn play_stolen_parts(
    _: &Card,
    game_state: &mut GameState,
    player_index: usize,
    req: &Request<Body>,
) {
    let target = header_value(req, "target").unwrap();
    game_state.damage_player(game_state.player_index_from_name(&target).unwrap(), 2);
    game_state.players[player_index].health += 1;
}
fn play_dent(_: &Card, game_state: &mut GameState, _: usize, req: &Request<Body>) {
    let target = header_value(req, "target").unwrap();
    game_state.damage_player(game_state.player_index_from_name(&target).unwrap(), 1);
}
