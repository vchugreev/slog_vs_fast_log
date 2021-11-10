use std::{
    fs::{self, OpenOptions},
    ops::{Add, Sub},
    time,
};

use chrono::{DateTime, Utc};
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use fast_log::{
    appender::{FastLogRecord, RecordFormat},
    filter::NoFilter,
    plugin::file::FileAppender,
};
use log::{info, Level};
use slog::{o, slog_info, Drain, Logger};
use slog_async::{self, OverflowStrategy};
use slog_term;

const SAMPLE_SIZE: usize = 500;
const MEASUREMENT_TIME: time::Duration = time::Duration::from_secs(30);
const CHANNEL_CAPACITY: usize = 1000;

const SLOG_FILE: &str = "slog_bench.log";
const FAST_LOG_FILE: &str = "fast_log_bench.log";

// Код на основе примера: https://github.com/slog-rs/misc/blob/master/examples/global_file_logger.rs
// + доп. настройки для асинхронного логирования и некоторые параметры типа емкости канала и стратегии переполнения
// Здесь также логер не инициализируется как глобальный логгер, чтобы не было конфликта с fast_log,
// который инициализируется как глобальный логер
fn create_slog() -> Logger {
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

    slog::Logger::root(drain, o!())
}

fn bench_logs(c: &mut Criterion) {
    let _ = fs::remove_file(FAST_LOG_FILE);

    let mut group = c.benchmark_group("fast log benchmark");
    group.sampling_mode(SamplingMode::Auto);
    group
        .sample_size(SAMPLE_SIZE)
        .measurement_time(MEASUREMENT_TIME);

    // Инициализируем fast_log, он будет привязан к глобальному логеру
    fast_log::init_custom_log(
        vec![Box::new(FileAppender::new(FAST_LOG_FILE))],
        CHANNEL_CAPACITY,
        Level::Info,
        Box::new(NoFilter {}),
        Box::new(LogRecord::new()),
    )
    .unwrap();

    // Создаем экземпляр slog-а, он сам по себе и вывод в файл осуществляется через его вспомогательные макросы,
    // типа slog_info
    let logger = create_slog();

    group.bench_function("fast_log", |b| b.iter(|| info!("===")));
    group.bench_function("slog", |b| b.iter(|| slog_info!(logger, "===")));

    group.finish();
}

// Вспомогательная структура для своего формата вывода DataTime в записях лога
// Формат сделан такой же как в slog-е, пример: Nov 10 11:32:23.429 INFO ===
// По умолчанию у fast_log-а другой формат: 2021-11-10 11:37:37.521506878 UTC    INFO fast_log::common - ===
// Чтобы сравнение было корректным, нужно писать в логи один и тот же объем данных.

pub struct LogRecord {
    pub duration: chrono::Duration,
}

impl RecordFormat for LogRecord {
    fn do_format(&self, arg: &mut FastLogRecord) {
        let data;
        let now: DateTime<Utc> = chrono::DateTime::from(arg.now);
        let now = now.add(self.duration);
        let now = now.format("%b %d %T%.3f").to_string();
        match arg.level {
            Level::Debug | Level::Warn | Level::Error => {
                data = format!(
                    "{} {} {} - {}  {}\n",
                    &now,
                    arg.level,
                    arg.module_path,
                    arg.args,
                    arg.format_line()
                );
            }
            _ => {
                data = format!("{} {} {}\n", &now, arg.level, arg.args);
            }
        }
        arg.formated = data;
    }
}

impl LogRecord {
    pub fn new() -> LogRecord {
        let utc = chrono::Utc::now().naive_utc();
        let tz = chrono::Local::now().naive_local();
        let d = tz.sub(utc);
        Self { duration: d }
    }
}

criterion_group!(benches, bench_logs);
criterion_main!(benches);
