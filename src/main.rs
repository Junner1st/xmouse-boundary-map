mod config;
mod geometry;
mod mapper;
mod x11_backend;

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

use crate::config::AppConfig;
use crate::mapper::{BoundaryMapper, MapOutcome};
use crate::x11_backend::X11Backend;

#[derive(Debug, Parser)]
#[command(version, about = "Map pointer crossings between differently sized X11 monitors")]
struct Cli {
    /// TOML config path. If omitted, adjacent horizontal monitors are auto-mapped.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Print detected monitors and exit.
    #[arg(long)]
    list_monitors: bool,

    /// Override poll interval in milliseconds.
    #[arg(long)]
    poll_ms: Option<u64>,

    /// Allow warping while mouse buttons are held. Default is to ignore drag.
    #[arg(long)]
    map_drag: bool,

    /// Log intended warps without moving the pointer.
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();
    let mut config = AppConfig::load_optional(cli.config.as_deref())?;
    if let Some(poll_ms) = cli.poll_ms {
        config.poll_interval_ms = poll_ms;
    }
    if cli.map_drag {
        config.ignore_drag = false;
    }

    let backend = X11Backend::connect().context("connect to X11")?;
    let monitors = backend.monitors().context("read XRandR monitor layout")?;

    if cli.list_monitors {
        for monitor in &monitors {
            println!(
                "{}: {}x{}+{}+{} ({}x{} mm)",
                monitor.name,
                monitor.width,
                monitor.height,
                monitor.x,
                monitor.y,
                monitor.mm_width.unwrap_or_default(),
                monitor.mm_height.unwrap_or_default()
            );
        }
        return Ok(());
    }

    if monitors.len() < 2 {
        anyhow::bail!("need at least two active monitors");
    }

    let mapper = BoundaryMapper::new(monitors, config.edge_mappings()?);
    if mapper.edge_count() == 0 {
        anyhow::bail!("no horizontal monitor edges found; provide --config with [[edge]] entries");
    }

    info!("active edges: {}", mapper.edge_count());
    for edge in mapper.edges() {
        info!(
            "edge {:?}: {} -> {} using {:?}",
            edge.side, edge.from, edge.to, edge.mode
        );
    }

    let poll_interval = Duration::from_millis(config.poll_interval_ms.max(1));
    let mut previous = backend.query_pointer().context("read initial pointer")?;
    let mut last_warp = Instant::now() - config.warp_cooldown();

    loop {
        thread::sleep(poll_interval);

        let pointer = match backend.query_pointer() {
            Ok(pointer) => pointer,
            Err(error) => {
                warn!(?error, "failed to query pointer");
                continue;
            }
        };

        let allow_drag = !config.ignore_drag || !pointer.buttons_down;
        if !allow_drag {
            previous = pointer;
            continue;
        }

        match mapper.map_crossing(&previous, &pointer) {
            MapOutcome::Warp(target) if last_warp.elapsed() >= config.warp_cooldown() => {
                if cli.dry_run {
                    info!("dry-run warp to {},{}", target.x, target.y);
                } else if let Err(error) = backend.warp_pointer(target) {
                    warn!(?error, "failed to warp pointer");
                    previous = pointer;
                    continue;
                } else {
                    debug!("warped pointer to {},{}", target.x, target.y);
                }
                last_warp = Instant::now();
                previous = pointer.with_position(target);
            }
            MapOutcome::Warp(_) => {
                previous = pointer;
            }
            MapOutcome::Noop => {
                previous = pointer;
            }
        }
    }
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
