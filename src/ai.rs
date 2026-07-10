use crate::structs::{GameState, Card, CardType, ShootingType};
use crate::{
    card_stays_active, remove_matching_cards, take_post_play_messages_since, log_play_event,
    blocking_calculated_activation_filter
};
use hyper::{Request, Body};
use rand::{Rng, thread_rng};

pub fn is_ai_player(name: &str) -> bool {
    name.to_lowercase().starts_with("ai ") || name.to_lowercase() == "ai"
}



pub fn process_recycle_choices_for_ais(game: &mut GameState) {
    let mut progress = true;
    while progress {
        progress = false;
        if let Some(mut pending) = game.pending_recycle.take() {
            // First, process any AI players who need to discard.
            let mut ai_to_discard = None;
            for player_name in &pending.awaiting_discards {
                if is_ai_player(player_name) {
                    ai_to_discard = Some(player_name.clone());
                    break;
                }
            }

            if let Some(ai_name) = ai_to_discard {
                if let Some(player_index) = game.player_index_from_name(&ai_name) {
                    let hand = &mut game.players[player_index].hand;
                    if !hand.is_empty() {
                        let discard_idx = thread_rng().gen_range(0..hand.len());
                        let discarded_card = hand.remove(discard_idx);
                        let card_name = discarded_card.name.clone();
                        pending.cards.push(discarded_card);
                        game.push_server_chat_message(format!(
                            "{} discarded {} for Recycle",
                            ai_name, card_name
                        ));
                    }
                }
                pending.awaiting_discards.retain(|p| p != &ai_name);
                game.pending_recycle = Some(pending);
                progress = true;
                continue;
            }

            // Next, if there are no more awaiting discards and the recycler is an AI, perform the recycle.
            if pending.awaiting_discards.is_empty() && is_ai_player(&pending.player) {
                let recycler = pending.player.clone();
                let mut recycled_card = None;
                if !pending.cards.is_empty() {
                    let choose_idx = thread_rng().gen_range(0..pending.cards.len());
                    recycled_card = Some(pending.cards.remove(choose_idx));
                }

                // Return remaining cards to discard pile.
                let remaining_cards = pending.cards;
                game.discard_pile.extend(remaining_cards);

                if let Some(card) = recycled_card {
                    let card_name = card.name.clone();
                    if let Some(recycler_idx) = game.player_index_from_name(&recycler) {
                        game.players[recycler_idx].hand.push(card);
                    }
                    game.push_server_chat_message(format!(
                        "{} recycled {}",
                        recycler, card_name
                    ));
                }

                game.pending_recycle = None;
                progress = true;
            } else {
                // Not an AI recycle choice, put it back
                game.pending_recycle = Some(pending);
            }
        }

    }
}

pub fn run_ai_turns(game: &mut GameState) {
    while game.alive_player_count() > 1 {
        // Run recycle choices for AIs in case there's a pending recycle.
        process_recycle_choices_for_ais(game);
        if game.pending_recycle.is_some() {
            // We cannot proceed with regular turns if we are waiting for human players to choose recycle discards.
            break;
        }

        let current_player = &game.players[game.current_turn_player];
        if is_ai_player(&current_player.name) {
            run_single_ai_turn(game);
            
            // Wait, after running the turn, check if the game ended.
            if game.alive_player_count() <= 1 {
                break;
            }
            
            // Advance turn
            let ended_turn_player_name = game.players[game.current_turn_player].name.clone();
            game.push_server_chat_message(format!(
                "{} ended their turn",
                ended_turn_player_name
            ));
            game.next_turn();
        } else {
            break;
        }
    }
}

fn is_healing_card(card: &Card) -> bool {
    ["repair", "repair-kit", "new-model"].contains(&card.id.as_str())
}

fn run_single_ai_turn(game: &mut GameState) {
    let current_player_name = game.players[game.current_turn_player].name.clone();

    // 1. Process any active calculated shootings first.
    loop {
        let ready_index = game.active_cards.active_calculated_shootings.iter().position(|active| {
            active.owner == current_player_name && active.turns_remaining == 0
        });

        let Some(active_index) = ready_index else {
            break;
        };

        let active = game.active_cards.active_calculated_shootings.remove(active_index);
        let card_id = active.card_played.id.clone();

        let mut req = Request::new(Body::empty());
        let targets = get_ai_targets(game, &current_player_name);

        if card_id == "decision-missile" {
            req.headers_mut().insert("decision_action", hyper::header::HeaderValue::from_static("draw"));
        } else if card_id == "multi-strike" {
            let allocations_str = if targets.is_empty() {
                r#"[]"#.to_string()
            } else if targets.len() == 1 {
                format!(
                    r#"[ {{"target": "{}", "damage": 1}}, {{"target": "{}", "damage": 6}} ]"#,
                    current_player_name, targets[0]
                )
            } else {
                format!(
                    r#"[ {{"target": "{}", "damage": 4}}, {{"target": "{}", "damage": 3}} ]"#,
                    targets[0], targets[1]
                )
            };
            req.headers_mut().insert("multistrike_allocations", hyper::header::HeaderValue::from_str(&allocations_str).unwrap());
        } else {
            if !targets.is_empty() {
                req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(&targets[0]).unwrap());
            }
        }

        if let Some((filter_owner, blocking_filter_name)) = blocking_calculated_activation_filter(game, &active, &req) {
            game.push_server_chat_message(format!(
                "{}'s {} blocked {}'s {}",
                filter_owner, blocking_filter_name, current_player_name, active.card_played.name
            ));
            game.discard_pile.push(active.card_played);
        } else {
            (active.when_done)(game, &active, &req);
            if card_id == "multi-strike" {
                game.turn_state.shooting_locked = true;
            }
            game.discard_pile.push(active.card_played);
        }
    }

    // 2. Play cards from hand.
    let mut played_any = true;
    while played_any && game.alive_player_count() > 1 {
        played_any = false;
        
        // In case recycle was played, we must immediately yield to process recycle.
        if game.pending_recycle.is_some() {
            break;
        }

        let player_index = game.current_turn_player;
        if game.players[player_index].name != current_player_name {
            break;
        }
        
        let hand_len = game.players[player_index].hand.len();
        
        for i in 0..hand_len {
            let card = game.players[player_index].hand[i].clone();
            
            // Build request headers for this card
            let mut req = Request::new(Body::empty());
            
            // Set headers based on what the card needs
            if is_healing_card(&card) {
                let target = if game.game_settings.play_heal_on_others {
                    // Find friendly target (alive player with lowest health, or self)
                    let mut best_target = current_player_name.clone();
                    let mut min_health = game.game_settings.max_health;
                    for p in &game.players {
                        if p.health > 0 && p.health < min_health {
                            min_health = p.health;
                            best_target = p.name.clone();
                        }
                    }
                    best_target
                } else {
                    current_player_name.clone()
                };
                req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(&target).unwrap());
            } else if card.id == "crack" || card.id == "dent" || card.id == "stolen-parts" {
                let targets = get_ai_attack_targets(game, &current_player_name, ShootingType::Quick);
                if let Some(target) = targets.first() {
                    req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(target).unwrap());
                }
            } else if card.id == "big-bomb" || card.id == "small-bomb" || card.id == "nuke" {
                let targets = get_ai_attack_targets(game, &current_player_name, ShootingType::Boom);
                if let Some(target) = targets.first() {
                    req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(target).unwrap());
                }
            } else if card.id == "health-hazard" {
                let targets = get_ai_targets(game, &current_player_name);
                let target = if let Some(t) = targets.first() {
                    t.clone()
                } else {
                    current_player_name.clone()
                };
                req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(&target).unwrap());
            } else if card.id == "steal" {
                let targets = get_ai_steal_targets(game, &current_player_name);
                if let Some(target) = targets.first() {
                    req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(target).unwrap());
                }
            } else if card.id == "airstrike" {
                let targets = get_ai_targets(game, &current_player_name);
                if let Some(target) = targets.first() {
                    req.headers_mut().insert("target", hyper::header::HeaderValue::from_str(target).unwrap());
                    let target_player = game.player_by_name(target).unwrap();
                    let target_hand_len = target_player.hand.len();
                    let action = if target_hand_len > 4 {
                        "reduce_to_three"
                    } else {
                        "discard_two"
                    };
                    req.headers_mut().insert("airstrike_action", hyper::header::HeaderValue::from_static(action));
                }
            } else if card.id == "new-model" {
                if let Some((c1, c2)) = find_two_shooting_cards_to_discard(&game.players[player_index].hand, i) {
                    req.headers_mut().insert("discardcard", hyper::header::HeaderValue::from_str(&c1).unwrap());
                    req.headers_mut().insert("discardcardtwo", hyper::header::HeaderValue::from_str(&c2).unwrap());
                }
            } else if card.id == "helpful-hand" {
                if let Some(c) = find_one_card_to_discard(&game.players[player_index].hand, i) {
                    req.headers_mut().insert("discardcard", hyper::header::HeaderValue::from_str(&c).unwrap());
                }
            } else if card.id == "distractor-missile" {
                if let Some(qc) = find_opponent_active_calculated_shooting(game, &current_player_name) {
                    req.headers_mut().insert("queuedcard", hyper::header::HeaderValue::from_str(&qc).unwrap());
                }
            } else if card.id == "cold-war" {
                if let Some(qc) = find_any_active_calculated_shooting(game) {
                    req.headers_mut().insert("queuedcard", hyper::header::HeaderValue::from_str(&qc).unwrap());
                }
            } else if card.id == "firing-filter" {
                let mut choose_type = "Quick";
                for t in &["Quick", "Calculated", "Boom"] {
                    let st = match *t {
                        "Quick" => ShootingType::Quick,
                        "Calculated" => ShootingType::Calculated,
                        "Boom" => ShootingType::Boom,
                        _ => ShootingType::Quick,
                    };
                    if !game.active_cards.active_firing_filters.iter().any(|f| f.owner == current_player_name && f.filter_type == st) {
                        choose_type = t;
                        break;
                    }
                }
                req.headers_mut().insert("firing_filter_type", hyper::header::HeaderValue::from_static(choose_type));
            } else if card.id == "radar" {
                let card_count = game.alive_player_count().min(game.draw_pile.len());
                let order: Vec<usize> = (0..card_count).collect();
                let order_str = serde_json::to_string(&order).unwrap();
                req.headers_mut().insert("radar_order", hyper::header::HeaderValue::from_str(&order_str).unwrap());
            }

            // Check if card can be played
            if (card.can_be_played)(&card, game, &req) {
                // Play card!
                let hand_before = game.players[player_index].hand.clone();
                let current_player_health_before = game.players[player_index].health;
                let target_name = req.headers().get("target").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
                let (target_health_before, target_hand_before) = target_name
                    .as_ref()
                    .and_then(|name| {
                        game.player_by_name(name)
                            .map(|player| (Some(player.health), Some(player.hand.clone())))
                    })
                    .unwrap_or((None, None));

                let discard_pile_len_before_play = game.discard_pile.len();
                game.death_discard_cards.clear();
                
                game.players[player_index].hand.remove(i);
                
                let chat_messages_len_before_play = game.chat_messages.len();
                (card.play)(&card, game, player_index, &req);
                let post_play_messages = take_post_play_messages_since(game, chat_messages_len_before_play);

                let safe_start = discard_pile_len_before_play.min(game.discard_pile.len());
                let mut discarded_cards: Vec<Card> = game.discard_pile[safe_start..].to_vec();
                let death_discard_cards = std::mem::take(&mut game.death_discard_cards);
                remove_matching_cards(&mut discarded_cards, &death_discard_cards);

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
                
                let stays_active = card_stays_active(&card);
                if !stays_active {
                    game.discard_pile.push(card.clone());
                    discarded_cards.push(card.clone());
                }

                let current_player_health_after = game
                    .player_by_name(&current_player_name)
                    .map(|player| player.health)
                    .unwrap_or(0);

                let target_hand_after = target_name
                    .as_ref()
                    .and_then(|name| game.player_by_name(name).map(|player| player.hand.clone()));

                log_play_event(
                    game,
                    &game.id.clone(),
                    &current_player_name,
                    &card,
                    &req,
                    &hand_before,
                    "played",
                    &discarded_cards,
                    None,
                    stays_active,
                    current_player_health_before,
                    current_player_health_after,
                    target_name.clone(),
                    target_health_before,
                    target_hand_before.as_deref(),
                    target_hand_after.as_deref(),
                );
                game.chat_messages.extend(post_play_messages);
                played_any = true;
                break;
            }
        }
    }
}

fn get_ai_targets(game: &GameState, current_player_name: &str) -> Vec<String> {
    game.players.iter()
        .filter(|p| p.health > 0 && p.name != current_player_name)
        .map(|p| p.name.clone())
        .collect()
}

fn get_ai_attack_targets(game: &GameState, current_player_name: &str, shooting_type: ShootingType) -> Vec<String> {
    let mut targets = Vec::new();
    for p in &game.players {
        if p.health > 0 && p.name != current_player_name {
            let blocked = game.active_cards.active_firing_filters.iter().any(|filter| {
                filter.owner == p.name && filter.filter_type == shooting_type
            });
            if !blocked {
                targets.push(p.name.clone());
            }
        }
    }
    targets
}

fn get_ai_steal_targets(game: &GameState, current_player_name: &str) -> Vec<String> {
    let mut opponents: Vec<&crate::structs::Player> = game.players.iter()
        .filter(|p| p.health > 0 && p.name != current_player_name && !p.hand.is_empty())
        .collect();
    opponents.sort_by_key(|p| std::cmp::Reverse(p.hand.len()));
    opponents.into_iter().map(|p| p.name.clone()).collect()
}

fn find_two_shooting_cards_to_discard(hand: &[Card], exclude_idx: usize) -> Option<(String, String)> {
    let shooting_indices: Vec<usize> = hand.iter().enumerate()
        .filter(|(idx, card)| *idx != exclude_idx && card.card_type.is_shooting())
        .map(|(idx, _)| idx)
        .collect();
    if shooting_indices.len() >= 2 {
        Some((
            format!("{}:{}", shooting_indices[0], hand[shooting_indices[0]].id),
            format!("{}:{}", shooting_indices[1], hand[shooting_indices[1]].id)
        ))
    } else {
        None
    }
}

fn find_one_card_to_discard(hand: &[Card], exclude_idx: usize) -> Option<String> {
    hand.iter().enumerate()
        .find(|(idx, _)| *idx != exclude_idx)
        .map(|(idx, card)| format!("{}:{}", idx, card.id))
}

fn find_opponent_active_calculated_shooting(game: &GameState, current_player_name: &str) -> Option<String> {
    game.active_cards.active_calculated_shootings.iter().enumerate()
        .find(|(_, shooting)| shooting.owner != current_player_name)
        .map(|(idx, shooting)| format!("{}:{}", idx, shooting.card_played.id))
}

fn find_any_active_calculated_shooting(game: &GameState) -> Option<String> {
    game.active_cards.active_calculated_shootings.iter().enumerate()
        .next()
        .map(|(idx, shooting)| format!("{}:{}", idx, shooting.card_played.id))
}
