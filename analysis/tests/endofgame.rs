use analysis::endofgame;
use pretty_assertions::assert_eq;

#[test]
fn nuke() {
    let input_bytes = include_bytes!("../../testfiles/nuke.dem");

    let result = endofgame::parse(input_bytes).unwrap();

    let expected = endofgame::EndOfGame {
        map: "de_nuke".to_owned(),
        players: vec![
            (
                endofgame::PlayerInfo {
                    name: "Excel".to_owned(),
                    steam_id: "76561198236134832".to_owned(),
                    team: 2,
                    color: 0,
                    ingame_id: 0,
                },
                endofgame::PlayerStats {
                    kills: 28,
                    deaths: 11,
                    damage: 2504,
                    team_damage: 0,
                    self_damage: 0,
                    assists: 4,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "Der Porzellan König".to_owned(),
                    steam_id: "76561198301388087".to_owned(),
                    team: 2,
                    color: 2,
                    ingame_id: 1,
                },
                endofgame::PlayerStats {
                    kills: 15,
                    deaths: 12,
                    damage: 1827,
                    team_damage: 4,
                    self_damage: 0,
                    assists: 6,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "Crippled Hentai addict".to_owned(),
                    steam_id: "76561198386810758".to_owned(),
                    team: 2,
                    color: 3,
                    ingame_id: 2,
                },
                endofgame::PlayerStats {
                    kills: 11,
                    deaths: 16,
                    damage: 1394,
                    team_damage: 13,
                    self_damage: 0,
                    assists: 5,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "Skalla_xD".to_owned(),
                    steam_id: "76561199014043225".to_owned(),
                    team: 2,
                    color: 1,
                    ingame_id: 3,
                },
                endofgame::PlayerStats {
                    kills: 11,
                    deaths: 15,
                    damage: 1331,
                    team_damage: 0,
                    self_damage: 0,
                    assists: 3,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "xTee".to_owned(),
                    steam_id: "76561199132258707".to_owned(),
                    team: 2,
                    color: 4,
                    ingame_id: 4,
                },
                endofgame::PlayerStats {
                    kills: 9,
                    deaths: 17,
                    damage: 1148,
                    team_damage: 0,
                    self_damage: 34,
                    // TODO
                    // Leetify says 2, my calculations say 3
                    assists: 3,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "cute".to_owned(),
                    steam_id: "76561197966517722".to_owned(),
                    team: 3,
                    color: 3,
                    ingame_id: 5,
                },
                endofgame::PlayerStats {
                    kills: 17,
                    deaths: 16,
                    damage: 2143,
                    team_damage: 109,
                    self_damage: 5,
                    assists: 7,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "zodiac".to_owned(),
                    steam_id: "76561198872143644".to_owned(),
                    team: 3,
                    color: 4,
                    ingame_id: 6,
                },
                endofgame::PlayerStats {
                    kills: 7,
                    deaths: 15,
                    damage: 844,
                    team_damage: 100,
                    self_damage: 0,
                    assists: 4,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "IReLaX exe".to_owned(),
                    steam_id: "76561199077629121".to_owned(),
                    team: 3,
                    color: 2,
                    ingame_id: 7,
                },
                endofgame::PlayerStats {
                    kills: 13,
                    deaths: 17,
                    damage: 1423,
                    team_damage: 44,
                    self_damage: 4,
                    assists: 6,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "Haze".to_owned(),
                    steam_id: "76561198375555469".to_owned(),
                    team: 3,
                    color: 0,
                    ingame_id: 8,
                },
                endofgame::PlayerStats {
                    kills: 19,
                    deaths: 15,
                    damage: 1512,
                    team_damage: 31,
                    self_damage: 0,
                    // TODO
                    // Leetify says 3, my calc says 4
                    assists: 4,
                },
            ),
            (
                endofgame::PlayerInfo {
                    name: "Know_Name".to_owned(),
                    steam_id: "76561198119236104".to_owned(),
                    team: 3,
                    color: 1,
                    ingame_id: 9,
                },
                endofgame::PlayerStats {
                    kills: 14,
                    deaths: 16,
                    damage: 1431,
                    team_damage: 68,
                    self_damage: 0,
                    assists: 4,
                },
            ),
        ],
    };

    // TODO
    // Add stats for rest of players

    assert_eq!(result, expected);
}
