use std::fmt::Display;
use std::fmt::Formatter;
use std::num::NonZero;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use futures::future::try_join_all;
use governor::Jitter;
use governor::Quota;
use governor::clock::DefaultClock;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use hdrhistogram::Histogram;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use tracing::Span;
use tracing::error;
use tracing::info;
use tracing::instrument;

use crate::Bench;
use crate::Workload;
use crate::client::Client;

#[instrument(skip_all)]
pub async fn benchmark<C>(client: C, bench: Bench) -> anyhow::Result<Report>
where
    C: Client + Clone + Send + 'static,
{
    let Bench {
        workload,
        workers,
        rate,
        duration,
        jitter,
        continue_on_error,
        warm_up,
    } = bench;
    let jitter = Duration::from_micros(jitter);
    let duration = Duration::from_secs(duration);
    let warm_up = Duration::from_secs(warm_up);

    let rate_limiter = Arc::new(governor::RateLimiter::direct(
        Quota::per_second(rate).allow_burst(NonZero::new(1).unwrap()),
    ));
    let ct = CancellationToken::new();
    let ct_warm_up = CancellationToken::new();

    info!("Warming up for {:.1} s", warm_up.as_secs_f64());
    let workers: Vec<_> = (client, rate_limiter)
        .multiply(workers)
        .map(|(client, rate_limiter)| {
            tokio::spawn(
                work(
                    workload,
                    ct.clone(),
                    client,
                    rate_limiter,
                    jitter,
                    continue_on_error,
                    ct_warm_up.clone(),
                )
                .instrument(Span::current()),
            )
        })
        .collect();
    let cancelled = tokio::select! {
        () = ct.cancelled() => {
            info!("Cancelled by worker");
            true
        },
        () = tokio::time::sleep(warm_up) => {
            info!("Completed warm up");
            false
        },
        result = tokio::signal::ctrl_c() => {
            result.expect("failed to listen for ctrl-c");
            info!("Cancelled by user");
            true
        },
    };
    ct_warm_up.cancel();
    let mut begin = Instant::now();
    if !cancelled {
        info!("Benchmarking for {:.1} s", duration.as_secs_f64());
        begin = Instant::now();
        tokio::select! {
            () = ct.cancelled() => {
                info!("Cancelled by worker");
            },
            () = tokio::time::sleep(duration) => {
                info!("Finished");
            },
            result = tokio::signal::ctrl_c() => {
                result.expect("failed to listen for ctrl-c");
                info!("Cancelled by user");
            },
        }
    }
    ct.cancel();
    let work_reports = try_join_all(workers).await.context("joining workers")?;
    let elapsed = begin.elapsed();

    Ok(Report::new(work_reports, elapsed))
}

#[instrument(skip_all)]
async fn work<C>(
    workload: Workload,
    ct: CancellationToken,
    mut client: C,
    rate_limiter: Arc<governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    jitter: Duration,
    continue_on_error: bool,
    ct_warm_up: CancellationToken,
) -> WorkReport
where
    C: Client,
{
    let jitter = Jitter::up_to(jitter);
    let mut histogram = Histogram::new(3).unwrap();
    let begin = Instant::now();
    let mut errors = 0;

    loop {
        tokio::select! {
            () = ct_warm_up.cancelled() => { break; },
            () = rate_limiter.until_ready_with_jitter(jitter) => {},
        }
        let result = match workload {
            Workload::Inty => client.inty().await.map(|_response| ()),
            Workload::Stringy => client.stringy().await.map(|_response| ()),
            Workload::Mixed => client.mixed().await.map(|_response| ()),
        };
        match result {
            Ok(()) => {}
            Err(error) => {
                errors += 1;
                if !continue_on_error {
                    error!(%error, error_dbg=?error, "error during warm-up");
                    ct.cancel();
                }
            }
        }
    }

    loop {
        tokio::select! {
            () = ct.cancelled() => { break; },
            () = rate_limiter.until_ready_with_jitter(jitter) => {},
        }
        let begin = Instant::now();
        let result = match workload {
            Workload::Inty => client.inty().await.map(|_response| ()),
            Workload::Stringy => client.stringy().await.map(|_response| ()),
            Workload::Mixed => client.mixed().await.map(|_response| ()),
        };
        let elapsed = u64::try_from(begin.elapsed().as_micros()).expect("gosh");
        match result {
            Ok(()) => {
                histogram.record(elapsed).unwrap();
            }
            Err(error) => {
                errors += 1;
                if !continue_on_error {
                    error!(%error, error_dbg=?error);
                    ct.cancel();
                }
            }
        }
    }
    WorkReport {
        histogram,
        errors,
        _duration: begin.elapsed(),
    }
}

struct WorkReport {
    histogram: Histogram<u64>,
    errors: usize,
    _duration: Duration,
}

pub struct Report {
    histogram: Histogram<u64>,
    errors: usize,
    duration: Duration,
}

impl Report {
    fn new(work_reports: impl IntoIterator<Item = WorkReport>, duration: Duration) -> Self {
        let mut histogram = Histogram::new(3).unwrap();
        let mut errors = 0;
        for work_report in work_reports {
            histogram += work_report.histogram;
            errors += work_report.errors;
        }
        Self {
            histogram,
            errors,
            duration,
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Latencies:")?;
        let quantiles = [0.50, 0.90, 0.95, 0.99, 1.00];
        let mut previous_microseconds = self.histogram.min();
        for quantile in quantiles {
            let microseconds = self.histogram.value_at_quantile(quantile);
            let samples_up_to = self.histogram.count_between(0, microseconds);
            writeln!(
                f,
                "\t{:>3}% (n={:>6}) [{:>7} us .. {:>7} us]",
                quantile * 100.0,
                samples_up_to,
                previous_microseconds,
                microseconds,
            )?;
            previous_microseconds = microseconds;
        }
        writeln!(f)?;
        let total_requests = usize::try_from(self.histogram.len()).unwrap() + self.errors;
        writeln!(f, "     Total requests: {total_requests}")?;
        writeln!(
            f,
            "            Elapsed: {:.2} s",
            self.duration.as_secs_f64()
        )?;
        #[allow(clippy::cast_precision_loss)]
        let requests_per_second = total_requests as f64 / self.duration.as_secs_f64();
        writeln!(f, "Requests per second: {requests_per_second:.2}")?;
        writeln!(f, "    Error responses: {}", self.errors)?;
        Ok(())
    }
}

trait Multiply: Clone {
    fn multiply(self, n: NonZeroUsize) -> Multiplied<Self> {
        Multiplied {
            remaining_clones: n.get() - 1,
            last: Some(self),
        }
    }
}

impl<T: Clone> Multiply for T {}

struct Multiplied<T> {
    remaining_clones: usize,
    last: Option<T>,
}

impl<T: Clone> Iterator for Multiplied<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_clones > 0 {
            self.remaining_clones -= 1;
            Some(
                self.last
                    .as_ref()
                    .expect("last is Some when remaining clones is positive")
                    .clone(),
            )
        } else {
            self.last.take()
        }
    }
}
