// Copyright (C) 2023 Ant group. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use protocols::empty;
use protocols::{mem_agent, mem_agent_ttrpc};
use share::option::{CompactSetOption, MemcgSetOption};
use structopt::StructOpt;
use ttrpc::r#async::Client;

#[derive(StructOpt, Debug)]
#[structopt(name = "mem-agent-ctl", about = "Memory agent controler")]
struct Opt {
    #[structopt(long, default_value = "unix:///var/run/mem-agent.sock")]
    addr: String,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "memcgstatus", about = "get memory cgroup status")]
    MemcgStatus,

    #[structopt(name = "memcgset", about = "set memory cgroup")]
    MemcgSet(MemcgSetOption),

    #[structopt(name = "compactset", about = "set compact")]
    CompactSet(CompactSetOption),

    #[structopt(name = "loglevelset", about = "set log level (trace, debug, info, warn, error, critical)")]
    LogLevelSet(LogLevelOption),
}

#[derive(Debug, StructOpt)]
struct LogLevelOption {
    #[structopt(name = "level")]
    level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    // setup client
    let c = Client::connect(&opt.addr).unwrap();
    let client = mem_agent_ttrpc::ControlClient::new(c.clone());

    match opt.command {
        Command::MemcgStatus => {
            let mss = client
                .memcg_status(ttrpc::context::with_timeout(0), &empty::Empty::new())
                .await
                .map_err(|e| anyhow!("client.memcg_status fail: {}", e))?;
            for mcg in mss.mem_cgroups {
                println!("{:?}", mcg);
                for (numa_id, n) in mcg.numa {
                    if let Some(t) = n.last_inc_time.into_option() {
                        println!("{} {:?}", numa_id, share::misc::timestamp_to_datetime(t)?);
                    }
                }
            }
        }

        Command::MemcgSet(c) => {
            let config = c.to_rpc_memcg_config();
            client
                .memcg_set(ttrpc::context::with_timeout(0), &config)
                .await
                .map_err(|e| anyhow!("client.memcg_status fail: {}", e))?;
        }

        Command::CompactSet(c) => {
            let config = c.to_rpc_compact_config();
            client
                .compact_set(ttrpc::context::with_timeout(0), &config)
                .await
                .map_err(|e| anyhow!("client.memcg_status fail: {}", e))?;
        }

        Command::LogLevelSet(c) => {
            let config = mem_agent::LogLevel {
                level: c.level.clone(),
                ..Default::default()
            };
            client
                .log_level_set(ttrpc::context::with_timeout(0), &config)
                .await
                .map_err(|e| anyhow!("client.memcg_status fail: {}", e))?;
        }
    }

    Ok(())
}
