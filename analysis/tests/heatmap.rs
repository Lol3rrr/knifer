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

#[test]
#[traced_test]
fn heatmap_dust2() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../testfiles/dust2.dem");
    dbg!(path);
    let input_bytes = std::fs::read(path).unwrap();

    let config = heatmap::Config { cell_size: 5.0 };
    let result = heatmap::parse(&config, &input_bytes).unwrap();

    assert_eq!(result.player_heatmaps.len(), result.player_info.len(), "Players: {:?}", result.player_heatmaps.keys().collect::<Vec<_>>());
    assert_eq!(
        result.player_info.len(),
        result.entity_to_player.len(),
        "Missing Entity-to-Player: {:?} - Missing Player-Info: {:?}",
        result.player_heatmaps.keys().filter(|entity| !result.entity_to_player.contains_key(*entity)).collect::<Vec<_>>(),
        result.player_info.iter().filter_map(|(user_id, info)| {
            if result.entity_to_player.values().any(|p| p == user_id) {
                return None;
            }

            Some(info)
        }).collect::<Vec<_>>(),
    );
}
