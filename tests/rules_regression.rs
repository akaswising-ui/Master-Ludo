use master_ludo::{Game, Phase, PlayerColor, YARD};

#[test]
fn invalid_dice_values_are_rejected_without_changing_the_turn() {
    for roll in [0, 7] {
        let mut game = Game::new(2);

        assert_eq!(
            game.roll_dice(roll),
            Err("Dice value must be between 1 and 6.")
        );
        assert_eq!(game.current_player, 0);
        assert_eq!(game.phase, Phase::AwaitRoll);
        assert_eq!(game.dice, None);
        assert_eq!(game.consecutive_sixes, 0);
    }
}

#[test]
fn a_second_roll_is_rejected_until_the_pending_token_moves() {
    let mut game = Game::new(2);
    game.roll_dice(6).unwrap();

    assert!(game.roll_dice(3).is_err());
    assert_eq!(game.current_player, 0);
    assert_eq!(game.phase, Phase::AwaitToken);
    assert_eq!(game.dice, Some(6));
    assert_eq!(game.consecutive_sixes, 1);
}

#[test]
fn an_illegal_token_choice_does_not_change_the_board_or_pending_roll() {
    let mut game = Game::new(2);
    game.players[0].tokens[0] = 2;
    game.roll_dice(1).unwrap();

    let before = game.players[0].tokens;
    assert_eq!(game.legal_moves(), vec![0]);
    assert!(matches!(
        game.move_token(1),
        Err("That token cannot move with this dice value.")
    ));

    assert_eq!(game.players[0].tokens, before);
    assert_eq!(game.phase, Phase::AwaitToken);
    assert_eq!(game.dice, Some(1));
}

#[test]
fn an_out_of_range_token_index_is_rejected_without_mutation() {
    let mut game = Game::new(2);
    game.roll_dice(6).unwrap();

    let before = game.players[0].tokens;
    assert!(matches!(game.move_token(4), Err("Invalid token index.")));
    assert_eq!(game.players[0].tokens, before);
    assert_eq!(game.phase, Phase::AwaitToken);
    assert_eq!(game.dice, Some(6));
}

#[test]
fn supported_player_counts_create_distinct_players_with_four_tokens_each() {
    for count in 2..=4 {
        let game = Game::new(count);

        assert_eq!(game.players.len(), count);
        for (index, player) in game.players.iter().enumerate() {
            assert_eq!(player.tokens, [YARD; 4]);
            assert!(game.players[..index]
                .iter()
                .all(|previous| previous.color != player.color));
        }
    }
}

#[test]
fn player_color_indices_match_the_four_board_entries() {
    assert_eq!(PlayerColor::ALL.map(PlayerColor::index), [0, 1, 2, 3]);
    assert_eq!(PlayerColor::ALL.map(PlayerColor::start_index), [0, 13, 26, 39]);
}
