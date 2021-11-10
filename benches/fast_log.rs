use std::fs;
use std::ops::{Add, Sub};

use chrono::{DateTime, Duration, Utc};
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use fast_log::{
    appender::{FastLogRecord, RecordFormat},
    filter::NoFilter,
    plugin::file::FileAppender,
};
use log::Level;

use common::{logging, CHANNEL_CAPACITY, MEASUREMENT_TIME, SAMPLE_SIZE};

mod common;

const FAST_LOG_FILE: &str = "fast_log_bench.log";

fn bench(c: &mut Criterion) {
    let _ = fs::remove_file(FAST_LOG_FILE);

    let mut group = c.benchmark_group("fast log benchmark");

    group.sampling_mode(SamplingMode::Linear);
    group
        .sample_size(SAMPLE_SIZE)
        .measurement_time(MEASUREMENT_TIME);

    // Инициализация логера ----
    fast_log::init_custom_log(
        vec![Box::new(FileAppender::new(FAST_LOG_FILE))],
        CHANNEL_CAPACITY,
        Level::Info,
        Box::new(NoFilter {}),
        Box::new(LogRecord::new()),
    )
    .unwrap();

    group.bench_function("fast_log", |b| b.iter(|| logging()));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);

// Вспомогательная структура для своего формата вывода DataTime в записях лога
// Формат сделан такой же как в slog-е, пример: Nov 10 11:32:23.429 INFO ===
// По умолчанию у fast_log-а другой формат: 2021-11-10 11:37:37.521506878 UTC    INFO fast_log::common - ===
// Чтобы сравнение было корректным, нужно писать в логи один и тот же объем данных.

pub struct LogRecord {
    pub duration: Duration,
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
