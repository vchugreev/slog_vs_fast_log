use std::fs::{self, OpenOptions};

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use slog::{o, Drain};
use slog_async::{self, OverflowStrategy};
use slog_scope;
use slog_stdlog;
use slog_term;

use common::{logging, CHANNEL_CAPACITY, MEASUREMENT_TIME, SAMPLE_SIZE};

mod common;

const SLOG_FILE: &str = "slog_bench.log";

fn bench(c: &mut Criterion) {
    let _ = fs::remove_file(SLOG_FILE);

    let mut group = c.benchmark_group("slog benchmark");

    group.sampling_mode(SamplingMode::Linear);
    group
        .sample_size(SAMPLE_SIZE)
        .measurement_time(MEASUREMENT_TIME);

    // Инициализация логера ----
    // Код на основе примера: https://github.com/slog-rs/misc/blob/master/examples/global_file_logger.rs
    // + доп. настойки для асинхронного логирования и некоторые параметры типа емкости канала и стратегии переполнения

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(SLOG_FILE)
        .unwrap();

    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();

    // Настраиваем емкость канала (такая же как в fast_log), а также включаем режим блокировки при переполнении
    // OverflowStrategy::Block т.к. подобное поведение наблюдается в fast_log
    let drain = slog_async::Async::new(drain)
        .chan_size(CHANNEL_CAPACITY)
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();

    let logger = slog::Logger::root(drain, o!());

    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    // ---

    group.bench_function("slog", |b| b.iter(|| logging()));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
