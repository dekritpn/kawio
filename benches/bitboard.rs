use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kawio::game::Game;

fn benchmark_flips(c: &mut Criterion) {
    let game = Game::new();
    c.bench_function("flips", |b| b.iter(|| game.flips(black_box(19))));
}

fn benchmark_legal_moves(c: &mut Criterion) {
    let game = Game::new();
    c.bench_function("legal_moves", |b| b.iter(|| game.legal_moves()));
}

criterion_group!(benches, benchmark_flips, benchmark_legal_moves);
criterion_main!(benches);