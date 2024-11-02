fn main() {
    divan::main();
}

#[divan::bench(args = ["dust2.dem", "inferno.dem", "nuke.dem"])]
fn heatmap(bencher: divan::Bencher, file: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../testfiles/")
        .join(file);
    let data = std::fs::read(path).unwrap();

    let config = analysis::heatmap::Config { cell_size: 2.0 };

    bencher.bench(|| analysis::heatmap::parse(divan::black_box(&config), divan::black_box(&data)));
}

#[divan::bench(args = ["dust2.dem", "inferno.dem", "nuke.dem"])]
fn perround(bencher: divan::Bencher, file: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../testfiles/")
        .join(file);
    let data = std::fs::read(path).unwrap();

    bencher.bench(|| analysis::perround::parse(divan::black_box(&data)));
}

#[divan::bench(args = ["dust2.dem", "inferno.dem", "nuke.dem"])]
fn endofgame(bencher: divan::Bencher, file: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../testfiles/")
        .join(file);
    let data = std::fs::read(path).unwrap();

    bencher.bench(|| analysis::endofgame::parse(divan::black_box(&data)));
}
