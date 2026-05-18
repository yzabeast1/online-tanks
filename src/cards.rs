use crate::structs::{Card, CardType, GameState, ShootingType};
use hyper::{Body, Request};

fn stub_play(_: &Card, _: &mut GameState, _: usize) {}

fn can_never_play(_: &Card, _: &GameState, _: &Request<Body>) -> bool {
    return false;
}

fn can_always_play(_: &Card, _: &GameState, _: &Request<Body>) -> bool {
    return true;
}

pub fn all_cards() -> Vec<Card> {
    let card_templates = vec![
        Card {
            name: "Repair".to_string(),
            id: "repair".to_string(),
            card_type: CardType::Support,
            can_be_played: can_play_repair,
            play: stub_play,
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
            play: stub_play,
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
            play: stub_play,
            count: 6,
        },
        Card {
            name: "Draw 2".to_string(),
            id: "draw-2".to_string(),
            card_type: CardType::Plus,
            can_be_played: can_always_play,
            play: stub_play,
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
            play: stub_play,
            count: 2,
        },
        Card {
            name: "Crack".to_string(),
            id: "crack".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_quick_shooting,
            play: stub_play,
            count: 5,
        },
        Card {
            name: "Dent".to_string(),
            id: "dent".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_quick_shooting,
            play: stub_play,
            count: 10,
        },
        Card {
            name: "Stolen Parts".to_string(),
            id: "stolen-parts".to_string(),
            card_type: CardType::Shooting(ShootingType::Quick),
            can_be_played: can_play_quick_shooting,
            play: stub_play,
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
            play: stub_play,
            count: 4,
        },
        Card {
            name: "Small Bomb".to_string(),
            id: "small-bomb".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_boom_shooting,
            play: stub_play,
            count: 10,
        },
        Card {
            name: "Landmine".to_string(),
            id: "landmine".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_boom_shooting,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Nuke".to_string(),
            id: "nuke".to_string(),
            card_type: CardType::Shooting(ShootingType::Boom),
            can_be_played: can_play_nuke,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "No Shooting".to_string(),
            id: "no-shooting".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_no_shooting,
            play: stub_play,
            count: 2,
        },
        Card {
            name: "More Ammo".to_string(),
            id: "more-ammo".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_event,
            play: stub_play,
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
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Lottery".to_string(),
            id: "lottery".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_lottery,
            play: stub_play,
            count: 1,
        },
        Card {
            name: "Health Hazard".to_string(),
            id: "health-hazard".to_string(),
            card_type: CardType::Event,
            can_be_played: can_play_event,
            play: stub_play,
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

fn can_play_firing_filter(_: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let firing_filter_type_str = headers
        .get("firing_filter_type")
        .and_then(|value| value.to_str().ok());

    let Some(firing_filter_type_str) = firing_filter_type_str else {
        return false;
    };

    let filter_type = match firing_filter_type_str {
        "Quick" => ShootingType::Quick,
        "Calculated" => ShootingType::Calculated,
        "Boom" => ShootingType::Boom,
        _ => return false,
    };

    // Check if the current player already has an active firing filter of this type
    let current_player_id = game_state.current_turn_player;
    for active_filter in &game_state.active_cards.active_firing_filters {
        if active_filter.owner == current_player_id && active_filter.filter_type == filter_type {
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

fn can_play_nuke(_card: &Card, game_state: &GameState, _req: &Request<Body>) -> bool {
    if game_state.turn_state.total_cards_played != 0 {
        return false;
    }
    return can_play_boom_shooting(_card, game_state, _req);
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
    let target_player_id = headers
        .get("target")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());

    if let Some(target_id) = target_player_id {
        for active_filter in &game_state.active_cards.active_firing_filters {
            if active_filter.owner == target_id && active_filter.filter_type == ShootingType::Quick
            {
                return false;
            }
        }
    }
    return can_play_shooting(game_state);
}

fn can_play_calculated_shooting(_: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_id = headers
        .get("target")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());

    if let Some(target_id) = target_player_id {
        for active_filter in &game_state.active_cards.active_firing_filters {
            if active_filter.owner == target_id
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
    let target_player_id = headers
        .get("target")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());

    if let Some(target_id) = target_player_id {
        for active_filter in &game_state.active_cards.active_firing_filters {
            if active_filter.owner == target_id && active_filter.filter_type == ShootingType::Boom {
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
    if game_state.turn_state.shooting_card_played
        >= game_state.turn_state.more_ammo_played as usize + 1
    {
        return false;
    }
    return true;
}

fn can_play_airstrike(_card: &Card, game_state: &GameState, req: &Request<Body>) -> bool {
    let headers = req.headers();
    let target_player_id = headers
        .get("target")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());

    let Some(target_player_id) = target_player_id else {
        return false;
    };

    let Some(target_player) = game_state.player_by_id(target_player_id) else {
        return false;
    };

    if target_player.hand.is_empty() {
        return false;
    }
    return can_play_event(_card, game_state, req);
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
    let target_player_id = headers
        .get("target")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());

    let Some(target_player_id) = target_player_id else {
        return false;
    };

    let Some(target_player) = game_state.player_by_id(target_player_id) else {
        return false;
    };

    if target_player.hand.is_empty() {
        return false;
    }
    return true;
}
