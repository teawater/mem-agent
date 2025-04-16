// Copyright (C) 2023 Ant group. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use mem_agent_lib::{agent, compact, memcg};
use protocols::mem_agent as rpc_mem_agent;
use protocols::{empty, mem_agent_ttrpc};
use slog_scope::{error, info};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use ttrpc::asynchronous::Server;
use ttrpc::error::Error;
use ttrpc::proto::Code;

#[derive(Debug)]
pub struct MyControl {
    agent: agent::MemAgent,
}

impl MyControl {
    #[allow(dead_code)]
    pub fn new(agent: agent::MemAgent) -> Self {
        Self { agent }
    }
}

fn mem_cgroup_to_rpc_mem_cgroup(path: &str, mcg: &memcg::MemCgroup) -> rpc_mem_agent::MemCgroup {
    rpc_mem_agent::MemCgroup {
        id: mcg.id as u32,
        ino: mcg.ino as u64,
        path: path.to_string(),
        numa: mcg
            .numa
            .iter()
            .map(|(numa_id, n)| {
                (
                    *numa_id,
                    rpc_mem_agent::StatusNuma {
                        last_inc_time: protobuf::MessageField::some(
                            crate::misc::datatime_to_timestamp(n.last_inc_time),
                        ),
                        max_seq: n.max_seq,
                        min_seq: n.min_seq,
                        run_aging_count: n.run_aging_count,
                        eviction_count: protobuf::MessageField::some(
                            rpc_mem_agent::EvictionCount {
                                page: n.eviction_count.page,
                                no_min_lru_file: n.eviction_count.no_min_lru_file,
                                min_lru_inc: n.eviction_count.min_lru_inc,
                                other_error: n.eviction_count.other_error,
                                error: n.eviction_count.error,
                                psi_exceeds_limit: n.eviction_count.psi_exceeds_limit,
                                ..Default::default()
                            },
                        ),
                        sleep_psi_exceeds_limit: n.sleep_psi_exceeds_limit,
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}

fn mem_cgroups_to_rpc_memcg_status(
    mgs: HashMap<String, memcg::MemCgroup>,
) -> rpc_mem_agent::MemcgStatusReply {
    let mem_cgroups: Vec<rpc_mem_agent::MemCgroup> = mgs
        .iter()
        .map(|(path, x)| mem_cgroup_to_rpc_mem_cgroup(path, &x))
        .collect();

    rpc_mem_agent::MemcgStatusReply {
        mem_cgroups,
        ..Default::default()
    }
}

fn rpc_memcg_single_config_to_single_option_config(
    sc: &rpc_mem_agent::MemcgSingleConfig,
) -> memcg::SingleOptionConfig {
    memcg::SingleOptionConfig {
        disabled: sc.disabled,
        swap: sc.swap,
        swappiness_max: sc.swappiness_max.map(|val| val as u8),
        period_secs: sc.period_secs,
        period_psi_percent_limit: sc.period_psi_percent_limit.map(|val| val as u8),
        eviction_psi_percent_limit: sc.eviction_psi_percent_limit.map(|val| val as u8),
        eviction_run_aging_count_min: sc.eviction_run_aging_count_min,
    }
}

fn rpc_memcg_config_item_to_cgroup_option_config(
    item: &rpc_mem_agent::MemcgConfigItem,
) -> memcg::CgroupOptionConfig {
    memcg::CgroupOptionConfig {
        path: item.path.clone(),
        numa_id: item.numa.clone(),
        no_subdir: item.no_subdir,
        config: rpc_memcg_single_config_to_single_option_config(&item.config),
    }
}

fn rpc_memcg_config_to_memcg_optionconfig(mc: &rpc_mem_agent::MemcgConfig) -> memcg::OptionConfig {
    let moc = memcg::OptionConfig {
        del: mc
            .del
            .iter()
            .map(|p| (p.path.clone(), p.numa.clone()))
            .collect(),
        add: mc
            .add
            .clone()
            .into_iter()
            .map(|item| rpc_memcg_config_item_to_cgroup_option_config(&item))
            .collect(),
        set: mc
            .set
            .clone()
            .into_iter()
            .map(|item| rpc_memcg_config_item_to_cgroup_option_config(&item))
            .collect(),
        default: rpc_memcg_single_config_to_single_option_config(&mc.default),
    };

    moc
}

fn compactconfig_to_compact_optionconfig(
    cc: &rpc_mem_agent::CompactConfig,
) -> compact::OptionConfig {
    let coc = compact::OptionConfig {
        disabled: cc.disabled,
        period_secs: cc.period_secs,
        period_psi_percent_limit: cc.period_psi_percent_limit.map(|val| val as u8),
        compact_psi_percent_limit: cc.compact_psi_percent_limit.map(|val| val as u8),
        compact_sec_max: cc.compact_sec_max,
        compact_order: cc.compact_order.map(|val| val as u8),
        compact_threshold: cc.compact_threshold,
        compact_force_times: cc.compact_force_times,
        ..Default::default()
    };

    coc
}

#[async_trait]
impl mem_agent_ttrpc::Control for MyControl {
    async fn memcg_status(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        _: empty::Empty,
    ) -> ::ttrpc::Result<rpc_mem_agent::MemcgStatusReply> {
        Ok(mem_cgroups_to_rpc_memcg_status(
            self.agent.memcg_status_async().await.map_err(|e| {
                let estr = format!("agent.memcg_status_async fail: {}", e);
                error!("{}", estr);
                Error::RpcStatus(ttrpc::get_status(Code::INTERNAL, estr))
            })?,
        ))
    }

    async fn memcg_set(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        mc: rpc_mem_agent::MemcgConfig,
    ) -> ::ttrpc::Result<empty::Empty> {
        self.agent
            .memcg_set_config_async(rpc_memcg_config_to_memcg_optionconfig(&mc))
            .await
            .map_err(|e| {
                let estr = format!("agent.memcg_set_config_async fail: {}", e);
                error!("{}", estr);
                Error::RpcStatus(ttrpc::get_status(Code::INTERNAL, estr))
            })?;
        Ok(empty::Empty::new())
    }

    async fn compact_set(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        cc: rpc_mem_agent::CompactConfig,
    ) -> ::ttrpc::Result<empty::Empty> {
        self.agent
            .compact_set_config_async(compactconfig_to_compact_optionconfig(&cc))
            .await
            .map_err(|e| {
                let estr = format!("agent.compact_set_config_async fail: {}", e);
                error!("{}", estr);
                Error::RpcStatus(ttrpc::get_status(Code::INTERNAL, estr))
            })?;
        Ok(empty::Empty::new())
    }
}

#[allow(dead_code)]
#[tokio::main]
pub async fn rpc_loop(agent: agent::MemAgent, addr: String) -> Result<()> {
    let path = addr
        .strip_prefix("unix://")
        .ok_or(anyhow!("format of addr {} is not right", addr))?;
    if std::path::Path::new(path).exists() {
        return Err(anyhow!("addr {} is exist", addr));
    }

    let control = MyControl::new(agent);
    let service = mem_agent_ttrpc::create_control(Arc::new(control));

    let mut server = Server::new()
        .bind(&addr)
        .map_err(|e| anyhow!("Server::new().bind {} fail: {}", addr, e))?
        .register_service(service);

    let metadata = fs::metadata(path).map_err(|e| anyhow!("fs::metadata {} fail: {}", path, e))?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)
        .map_err(|e| anyhow!("fs::set_permissions {} fail: {}", path, e))?;

    let mut interrupt = signal(SignalKind::interrupt())
        .map_err(|e| anyhow!("signal(SignalKind::interrupt()) fail: {}", e))?;
    let mut quit = signal(SignalKind::quit())
        .map_err(|e| anyhow!("signal(SignalKind::quit()) fail: {}", e))?;
    let mut terminate = signal(SignalKind::terminate())
        .map_err(|e| anyhow!("signal(SignalKind::terminate()) fail: {}", e))?;
    server
        .start()
        .await
        .map_err(|e| anyhow!("server.start() fail: {}", e))?;

    tokio::select! {
        _ = interrupt.recv() => {
            info!("mem-agent: interrupt shutdown");
        }

        _ = quit.recv() => {
            info!("mem-agent: quit shutdown");
        }

        _ = terminate.recv() => {
            info!("mem-agent: terminate shutdown");
        }
    };

    server
        .shutdown()
        .await
        .map_err(|e| anyhow!("server.shutdown() fail: {}", e))?;
    fs::remove_file(&path).map_err(|e| anyhow!("fs::remove_file {} fail: {}", path, e))?;

    Ok(())
}
