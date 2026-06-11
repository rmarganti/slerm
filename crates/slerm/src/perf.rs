use std::{env, sync::OnceLock, time::Duration};

static TERMINAL_PERF_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn terminal_perf_enabled() -> bool {
    *TERMINAL_PERF_ENABLED.get_or_init(|| {
        env::var("SLERM_TERMINAL_PERF")
            .map(|value| !matches!(value.as_str(), "" | "0" | "false" | "FALSE" | "off" | "OFF"))
            .unwrap_or(false)
    })
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TerminalDrainPerf {
    pub terminals: usize,
    pub changed_terminals: usize,
    pub bytes_read: usize,
    pub duration: Duration,
}

impl TerminalDrainPerf {
    pub fn record_terminal(&mut self, terminal: TerminalDrainPerf) {
        self.terminals += terminal.terminals;
        self.changed_terminals += terminal.changed_terminals;
        self.bytes_read += terminal.bytes_read;
        self.duration += terminal.duration;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TerminalFramePerf {
    pub drain: TerminalDrainPerf,
    pub snapshot_duration: Duration,
    pub rows_considered: usize,
    pub cells_considered: usize,
    pub render_items: usize,
    pub shape_line_calls: usize,
    pub prepaint_duration: Duration,
}

impl TerminalFramePerf {
    pub fn log_if_enabled(&self) {
        if terminal_perf_enabled() {
            eprintln!(
                "slerm terminal perf: prepaint={:.2?} drain={:.2?} drain_bytes={} drain_changed_terminals={} snapshot={:.2?} rows={} cells={} render_items={} shape_line_calls={}",
                self.prepaint_duration,
                self.drain.duration,
                self.drain.bytes_read,
                self.drain.changed_terminals,
                self.snapshot_duration,
                self.rows_considered,
                self.cells_considered,
                self.render_items,
                self.shape_line_calls,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_perf_records_terminal_metrics() {
        let mut aggregate = TerminalDrainPerf::default();
        aggregate.record_terminal(TerminalDrainPerf {
            terminals: 1,
            changed_terminals: 1,
            bytes_read: 12,
            duration: Duration::from_millis(2),
        });
        aggregate.record_terminal(TerminalDrainPerf {
            terminals: 1,
            changed_terminals: 0,
            bytes_read: 0,
            duration: Duration::from_millis(1),
        });

        assert_eq!(aggregate.terminals, 2);
        assert_eq!(aggregate.changed_terminals, 1);
        assert_eq!(aggregate.bytes_read, 12);
        assert_eq!(aggregate.duration, Duration::from_millis(3));
    }
}
