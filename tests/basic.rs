use gtest::{Program, System};
use pebbles_game::*;
use pebbles_game_io::*;

const USERS: &[u64] = &[33, 44, 55];
const VALUE: u128 = 1000000000000000000;

fn init_game(sys: &System, total: u32, turn_max: u32) {
    sys.init_logger();

    let game = Program::current_opt(sys);
    sys.mint_to(USERS[0], VALUE);
    game.send(
        USERS[0],
        PebblesInit {
            pebbles_count: total,
            max_pebbles_per_turn: turn_max,
            difficulty: DifficultyLevel::Easy,
        },
    );
    sys.run_next_block();

    let gm: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(gm.pebbles_count, total);
    assert_eq!(gm.max_pebbles_per_turn, turn_max);
    match gm.first_player {
        Player::User => assert_eq!(gm.pebbles_count, gm.pebbles_remaining),
        Player::Program => assert_eq!(gm.pebbles_count, gm.pebbles_remaining + gm.program_lastmove),
    }
}

#[test]
fn init_successed() {
    let sys = System::new();
    sys.init_logger();
    sys.mint_to(USERS[0], VALUE);

    let game = Program::current_opt(&sys);
    game.send(
        USERS[0],
        PebblesInit {
            pebbles_count: 10,
            max_pebbles_per_turn: 9,
            difficulty: DifficultyLevel::Easy,
        },
    );
    sys.run_next_block();
    let gm: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(gm.pebbles_count, 10);
    assert_eq!(gm.max_pebbles_per_turn, 9);
    assert_eq!(gm.difficulty, DifficultyLevel::Easy);
}

#[test]
fn user_move() {
    let sys = System::new();
    init_game(&sys, 101, 3);
    let game = sys.get_program(1).unwrap();
    let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
    let mut remaing = gmstate.pebbles_remaining;

    game.send(USERS[0], PebblesAction::Turn(1));
    let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(94, remaing - 1 - gmstate.program_lastmove);
    remaing = gmstate.pebbles_remaining;
    game.send(USERS[0], PebblesAction::Turn(2));
    let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(93, remaing - 2 - gmstate.program_lastmove);
    remaing = gmstate.pebbles_remaining;
    game.send(USERS[0], PebblesAction::Turn(3));
    let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(92, remaing - 3 - gmstate.program_lastmove);
}

#[test]
fn user_move_failed() {
    let sys = System::new();
    init_game(&sys, 5, 2);
    let game = sys.get_program(1).unwrap();

    game.send(USERS[0], PebblesAction::Turn(0));
    sys.run_next_block();
    game.send(USERS[0], PebblesAction::Turn(3));
    sys.run_next_block();
    let gm: PebbleGame = game.read_state(0).expect("Invalid state.");
    println!("gm: {:?}", gm);
}
#[test]
fn user_move_failed2() {
    let sys2 = System::new();
    init_game(&sys2, 3, 2);

    let game = sys2.get_program(1).unwrap();
    loop {
        let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
        if gmstate.program_lastmove == 2 {
            break;
        }
        game.send(
            USERS[0],
            PebblesAction::Restart {
                difficulty: DifficultyLevel::Easy,
                pebbles_count: 3,
                max_pebbles_per_turn: 2,
            },
        );
        sys2.run_next_block();
    }
    game.send(USERS[0], PebblesAction::Turn(2));
    sys2.run_next_block();
    let gm: PebbleGame = game.read_state(0).expect("Invalid state.");
    println!("gm: {:?}", gm);
}

#[test]
fn user_give_up() {
    let sys = System::new();
    init_game(&sys, 99, 3);
    let game = sys.get_program(1).unwrap();
    let game_state: PebbleGame = game.read_state(0).expect("Invalid state.");
    let remaining = game_state.pebbles_remaining;
    // TODO ddd program move action
    game.send(USERS[0], PebblesAction::GiveUp);
    sys.run_next_block();
    let game_state: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(
        game_state.pebbles_remaining,
        remaining - game_state.program_lastmove
    );
}

#[test]
fn winner() {
    let sys = System::new();
    init_game(&sys, 3, 1);
    let game = sys.get_program(1).unwrap();

    for _ in 0..100 {
        game.send(
            USERS[0],
            PebblesAction::Restart {
                difficulty: DifficultyLevel::Easy,
                pebbles_count: 3,
                max_pebbles_per_turn: 1,
            },
        );
        sys.run_next_block();
        let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
        let remaing = gmstate.pebbles_remaining;
        if remaing < 3 {
            game.send(USERS[0], PebblesAction::Turn(1));
            sys.run_next_block();
        } else {
            game.send(USERS[0], PebblesAction::Turn(1));
            sys.run_next_block();
            game.send(USERS[0], PebblesAction::Turn(1));
            sys.run_next_block();
        }
    }
    let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(gmstate.winner, Some(Player::Program));
}

#[test]
fn restart() {
    let sys = System::new();
    init_game(&sys, 3, 1);
    let game = sys.get_program(1).unwrap();
    game.send(
        USERS[0],
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 50,
            max_pebbles_per_turn: 3,
        },
    );
    sys.run_next_block();
    let gmstate: PebbleGame = game.read_state(0).expect("Invalid state.");
    assert_eq!(gmstate.pebbles_count, 50);
    assert_eq!(gmstate.max_pebbles_per_turn, 3);
}
