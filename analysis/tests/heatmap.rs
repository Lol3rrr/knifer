use analysis::heatmap;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn heatmap_nuke() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../testfiles/nuke.dem");
    dbg!(path);
    let input_bytes = std::fs::read(path).unwrap();

    let config = heatmap::Config { cell_size: 5.0 };
    let result = heatmap::parse(&config, &input_bytes).unwrap();

    assert_eq!(result.player_heatmaps.len(), result.player_info.len());
}

#[test]
#[traced_test]
fn heatmap_inferno() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../testfiles/inferno.dem");
    dbg!(path);
    let input_bytes = std::fs::read(path).unwrap();

    let config = heatmap::Config { cell_size: 5.0 };
    let result = heatmap::parse(&config, &input_bytes).unwrap();

    assert_eq!(result.player_heatmaps.len(), result.player_info.len(), "Players: {:?}", result.player_heatmaps.keys().collect::<Vec<_>>());
}
