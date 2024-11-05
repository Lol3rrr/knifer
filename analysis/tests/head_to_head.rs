use analysis::head_to_head;

use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
#[ignore = "Testing"]
fn head_to_head_nuke() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../testfiles/nuke.dem");
    dbg!(path);
    let input_bytes = std::fs::read(path).unwrap();

    let result = head_to_head::parse(&input_bytes).unwrap();

    let expected = head_to_head::Output {
        players: [(csdemo::UserId(0), csdemo::parser::Player {
            xuid: 0,
            name: "".to_owned(),
            team: 0,
            color: 0,
        })].into_iter().collect(),
        head_to_head: [
            (csdemo::UserId(0), HashMap::new()),
            (csdemo::UserId(1), HashMap::new()),
            (csdemo::UserId(2), HashMap::new()),
            (csdemo::UserId(3), HashMap::new()),
            (csdemo::UserId(4), HashMap::new()),
            (csdemo::UserId(5), HashMap::new()),
            (csdemo::UserId(6), HashMap::new()),
            (csdemo::UserId(7), HashMap::new()),
            (csdemo::UserId(8), HashMap::new()),
            (csdemo::UserId(9), HashMap::new()),
        ].into_iter().collect(),
    };

    dbg!(&expected, &result);
    assert_eq!(result, expected);

    todo!()
}
