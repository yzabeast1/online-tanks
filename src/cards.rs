use crate::structs::{Card, CardType, GameState, ShootingType};

fn stub_can_be_played(_: &Card, _: &GameState) -> bool {
    false
}

fn stub_play(_: &Card, _: &mut GameState, _: usize) {}

pub fn all_cards() -> Vec<Card> {
    vec![Card::new(
        "Dent".to_string(),
        "dent".to_string(),
        "cards/dent.png".to_string(),
        CardType::Shooting(ShootingType::Quick),
        can_play_quick_shooting,
        stub_play,
    )]
}

fn can_play_quick_shooting(card: &Card, game_state: &GameState) -> bool {
    if game_state.no_shooting_played_by != -1
        || game_state.current_turn_player as isize == game_state.no_shooting_played_by
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

fn can_play_event(card: &Card, game_state: &GameState) -> bool{
    if game_state.turn_state.event_card_played {
        return false;
    }
    return true;
}
