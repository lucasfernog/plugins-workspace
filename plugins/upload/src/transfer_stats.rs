// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::time::{Duration, Instant};

/// Tracks the cumulative transfer progress and the current transfer speed.
pub struct TransferStats {
    /// Bytes transferred since `period_start`.
    period_bytes: u64,
    /// When the current measurement period started.
    period_start: Instant,
    /// How long a measurement period lasts before the speed is recalculated.
    granularity: Duration,
    /// The transfer speed in bytes per second, measured over the last complete period.
    pub transfer_speed: u64,
    /// Cumulative total of all transferred data.
    pub total_transferred: u64,
}

impl TransferStats {
    /// Initializes a new TransferStats instance with the specified granularity in milliseconds.
    pub fn start(granularity: u32) -> Self {
        Self::start_at(granularity, Instant::now())
    }

    fn start_at(granularity: u32, now: Instant) -> Self {
        Self {
            period_bytes: 0,
            period_start: now,
            granularity: Duration::from_millis(granularity.into()),
            transfer_speed: 0,
            total_transferred: 0,
        }
    }

    /// Records the transfer of a data chunk and updates both transfer speed and total progress.
    pub fn record_chunk_transfer(&mut self, chunk_len: usize) {
        self.record_chunk_transfer_at(chunk_len, Instant::now());
    }

    fn record_chunk_transfer_at(&mut self, chunk_len: usize, now: Instant) {
        let chunk_len = chunk_len as u64;
        self.period_bytes += chunk_len;
        self.total_transferred += chunk_len;

        // Measure against the real elapsed time of the period (not a sum of per-chunk
        // millisecond intervals), so chunks that arrive less than a millisecond apart still count.
        let elapsed = now.saturating_duration_since(self.period_start);
        if elapsed >= self.granularity && !elapsed.is_zero() {
            self.transfer_speed = (self.period_bytes as f64 / elapsed.as_secs_f64()) as u64;
            self.period_bytes = 0;
            self.period_start = now;
        }
    }
}

// Provides a default implementation for TransferStats with a granularity of 500 milliseconds.
impl Default for TransferStats {
    fn default() -> Self {
        Self::start(500) // Default granularity is 500 ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_is_bytes_per_second() {
        let start = Instant::now();
        let mut stats = TransferStats::start_at(500, start);
        stats.record_chunk_transfer_at(1000, start + Duration::from_millis(100));
        assert_eq!(stats.transfer_speed, 0, "speed is only computed per period");
        stats.record_chunk_transfer_at(1000, start + Duration::from_millis(500));
        // 2000 bytes in 0.5 s
        assert_eq!(stats.transfer_speed, 4000);
        assert_eq!(stats.total_transferred, 2000);
    }

    #[test]
    fn sub_millisecond_chunks_are_counted() {
        let start = Instant::now();
        let mut stats = TransferStats::start_at(500, start);
        // 1000 chunks of 1 KiB, 0.5 ms apart: every per-chunk interval is below 1 ms
        for i in 1..=1000u64 {
            stats.record_chunk_transfer_at(1024, start + Duration::from_micros(i * 500));
        }
        // 1_024_000 bytes in 0.5 s
        assert_eq!(stats.transfer_speed, 2_048_000);
        assert_eq!(stats.total_transferred, 1_024_000);
    }

    #[test]
    fn slow_transfers_do_not_round_to_zero() {
        let start = Instant::now();
        let mut stats = TransferStats::start_at(500, start);
        // 100 bytes in 1 s is below 1 byte/ms, which used to truncate to 0
        stats.record_chunk_transfer_at(100, start + Duration::from_secs(1));
        assert_eq!(stats.transfer_speed, 100);
    }

    #[test]
    fn speed_is_measured_per_period() {
        let start = Instant::now();
        let mut stats = TransferStats::start_at(500, start);
        stats.record_chunk_transfer_at(10_000, start + Duration::from_secs(1));
        assert_eq!(stats.transfer_speed, 10_000);
        stats.record_chunk_transfer_at(500, start + Duration::from_secs(2));
        assert_eq!(stats.transfer_speed, 500);
        assert_eq!(stats.total_transferred, 10_500);
    }
}
