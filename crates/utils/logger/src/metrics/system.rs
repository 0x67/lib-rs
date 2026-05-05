use super::config::SystemMetricsConfig;
use sysinfo::{Networks, ProcessesToUpdate, System};
use tokio::task::JoinHandle;

pub(crate) fn spawn_system_metrics(
    meter: opentelemetry::metrics::Meter,
    config: SystemMetricsConfig,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let interval = config.interval();
        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();

        let pid = match sysinfo::get_current_pid() {
            Ok(pid) => pid,
            Err(e) => {
                eprintln!("Failed to get current PID for system metrics: {e}");
                return;
            }
        };

        #[cfg(feature = "jemalloc")]
        let jemalloc_instruments = super::jemalloc::record_jemalloc_metrics(&meter);

        let cpu_gauge = meter
            .f64_gauge("process.cpu.utilization")
            .with_description("Process CPU utilization percentage")
            .with_unit("percent")
            .build();

        let memory_gauge = meter
            .u64_gauge("process.memory.usage")
            .with_description("Process physical memory usage")
            .with_unit("By")
            .build();

        let virtual_memory_gauge = meter
            .u64_gauge("process.memory.virtual")
            .with_description("Process virtual memory usage")
            .with_unit("By")
            .build();

        let disk_read_gauge = meter
            .u64_gauge("process.disk.io.read")
            .with_description("Process cumulative disk read bytes")
            .with_unit("By")
            .build();

        let disk_write_gauge = meter
            .u64_gauge("process.disk.io.write")
            .with_description("Process cumulative disk write bytes")
            .with_unit("By")
            .build();

        let network_tx_gauge = meter
            .u64_gauge("system.network.io.transmit")
            .with_description("Total network bytes transmitted across all interfaces")
            .with_unit("By")
            .build();

        let network_rx_gauge = meter
            .u64_gauge("system.network.io.receive")
            .with_description("Total network bytes received across all interfaces")
            .with_unit("By")
            .build();

        loop {
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

            if let Some(process) = sys.process(pid) {
                cpu_gauge.record(f64::from(process.cpu_usage()), &[]);
                memory_gauge.record(process.memory(), &[]);
                virtual_memory_gauge.record(process.virtual_memory(), &[]);

                let disk = process.disk_usage();
                disk_read_gauge.record(disk.total_read_bytes, &[]);
                disk_write_gauge.record(disk.total_written_bytes, &[]);
            }

            networks.refresh(true);
            let (tx_total, rx_total) =
                networks
                    .iter()
                    .fold((0u64, 0u64), |(tx, rx), (_name, data)| {
                        (
                            tx.saturating_add(data.total_transmitted()),
                            rx.saturating_add(data.total_received()),
                        )
                    });
            network_tx_gauge.record(tx_total, &[]);
            network_rx_gauge.record(rx_total, &[]);

            #[cfg(feature = "jemalloc")]
            jemalloc_instruments.collect();

            tokio::time::sleep(interval).await;
        }
    })
}
