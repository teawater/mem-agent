// Copyright (C) 2023 Ant group. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use share::option::{CompactSetOption, MemcgSetupOption};
use slog::Level;
use slog_scope::{error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "mem-agent", about = "Memory agent")]
struct Opt {
    #[structopt(long, default_value = "unix:///var/run/mem-agent.sock")]
    addr: String,
    #[structopt(long)]
    log_file: Option<String>,
    #[structopt(long, default_value = "trace", parse(try_from_str = share::logger::parse_slog_level))]
    log_level: Level,
    #[structopt(flatten)]
    memcg: MemcgSetupOption,
    #[structopt(flatten)]
    compact: CompactSetOption,
}

fn main() -> Result<()> {
    // Check opt
    let opt = Opt::from_args();

    let _logger_guard = share::logger::setup_logging(&opt.log_file, opt.log_level)
        .map_err(|e| anyhow!("setup_logging fail: {}", e))?;

    let memcg_config = opt.memcg.to_mem_agent_memcg_config();
    let compact_config = opt.compact.to_mem_agent_compact_config();

    let (ma, _rt) = mem_agent_lib::agent::MemAgent::new(memcg_config, compact_config)
        .map_err(|e| anyhow!("MemAgent::new fail: {}", e))?;

    info!("MemAgent started");

    share::rpc::rpc_loop(ma, opt.addr).map_err(|e| {
        let estr = format!("rpc::rpc_loop fail: {}", e);
        error!("{}", estr);
        anyhow!("{}", estr)
    })?;

    Ok(())
}
