use analysis::perround;
use pretty_assertions::assert_eq;

#[test]
fn perround_nuke() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../testfiles/nuke.dem");
    dbg!(path);
    let input_bytes = std::fs::read(path).unwrap();

    let result = perround::parse(&input_bytes).unwrap();
    dbg!(&result);

    assert_eq!(21, result.rounds.len());
}
