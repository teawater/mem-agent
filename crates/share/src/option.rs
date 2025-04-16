// Copyright (C) 2024 Ant group. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use protocols::mem_agent as rpc;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Default)]
pub struct CgroupMemcgSetOption {
    memcg_path: String,
    memcg_numa_id: Vec<u32>,
    memcg_disabled: Option<bool>,
    memcg_swap: Option<bool>,
    memcg_swappiness_max: Option<u8>,
    memcg_period_secs: Option<u64>,
    memcg_period_psi_percent_limit: Option<u8>,
    memcg_eviction_psi_percent_limit: Option<u8>,
    memcg_eviction_run_aging_count_min: Option<u64>,
    no_subdir: Option<bool>,
}

impl FromStr for CgroupMemcgSetOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut cg = CgroupMemcgSetOption::default();

        for pair in s.split(',') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() != 2 {
                return Err(anyhow!("{} is invalid", pair));
            }
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "path" => cg.memcg_path = value.to_string(),
                "numa-id" => {
                    cg.memcg_numa_id = value
                        .split(':')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse::<u32>())
                        .collect::<Result<Vec<u32>, _>>()?;
                }
                "disabled" => cg.memcg_disabled = Some(value.parse::<bool>()?),
                "swap" => cg.memcg_swap = Some(value.parse::<bool>()?),
                "swappiness-max" => cg.memcg_swappiness_max = Some(value.parse::<u8>()?),
                "period-secs" => cg.memcg_period_secs = Some(value.parse::<u64>()?),
                "period-psi-percent-limit" => {
                    cg.memcg_period_psi_percent_limit = Some(value.parse::<u8>()?)
                }
                "eviction-psi-percent-limit" => {
                    cg.memcg_eviction_psi_percent_limit = Some(value.parse::<u8>()?)
                }
                "eviction-run-aging-count-min" => {
                    cg.memcg_eviction_run_aging_count_min = Some(value.parse::<u64>()?)
                }
                "no-subdir" => cg.no_subdir = Some(value.parse::<bool>()?),
                _ => return Err(anyhow!("{} is invalid", key)),
            }
        }

        if cg.memcg_path.is_empty() {
            return Err(anyhow!("path is required"));
        }

        Ok(cg)
    }
}

impl CgroupMemcgSetOption {
    fn to_rpc_memcg_config_item(&self) -> rpc::MemcgConfigItem {
        rpc::MemcgConfigItem {
            path: self.memcg_path.clone(),
            numa: self.memcg_numa_id.clone(),
            no_subdir: self.no_subdir,
            config: Some(rpc::MemcgSingleConfig {
                disabled: self.memcg_disabled,
                swap: self.memcg_swap,
                swappiness_max: self.memcg_swappiness_max.map(|v| v as u32),
                period_secs: self.memcg_period_secs,
                period_psi_percent_limit: self.memcg_period_psi_percent_limit.map(|v| v as u32),
                eviction_psi_percent_limit: self.memcg_eviction_psi_percent_limit.map(|v| v as u32),
                eviction_run_aging_count_min: self.memcg_eviction_run_aging_count_min,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct MemcgSetupOption {
    #[structopt(long)]
    memcg_disabled: Option<bool>,
    #[structopt(long)]
    memcg_swap: Option<bool>,
    #[structopt(long)]
    memcg_swappiness_max: Option<u8>,
    #[structopt(long)]
    memcg_period_secs: Option<u64>,
    #[structopt(long)]
    memcg_period_psi_percent_limit: Option<u8>,
    #[structopt(long)]
    memcg_eviction_psi_percent_limit: Option<u8>,
    #[structopt(long)]
    memcg_eviction_run_aging_count_min: Option<u64>,
    #[structopt(long)]
    memcg_cgroups: Vec<CgroupMemcgSetOption>,
}

macro_rules! set_fields {
    // Format：copy_fields!(source, target, [src_field => target_field, ...])
    ($source:expr, $target:expr, [$($src_field:ident => $target_field:ident),*]) => {
        $(
            // Each field
            if let Some(val) = &$source.$src_field {
                $target.$target_field = val.clone();
            }
        )*
    };
}

impl MemcgSetupOption {
    #[allow(dead_code)]
    pub fn to_mem_agent_memcg_config(&self) -> mem_agent_lib::memcg::Config {
        let mut config = mem_agent_lib::memcg::Config {
            ..Default::default()
        };

        set_fields!(self, config.default, [
            memcg_disabled => disabled,
            memcg_swap => swap,
            memcg_swappiness_max => swappiness_max,
            memcg_period_secs => period_secs,
            memcg_period_psi_percent_limit => period_psi_percent_limit,
            memcg_eviction_psi_percent_limit => eviction_psi_percent_limit,
            memcg_eviction_run_aging_count_min => eviction_run_aging_count_min
        ]);

        for cg in self.memcg_cgroups.iter() {
            let mut cc = mem_agent_lib::memcg::CgroupConfig::default();
            if let Some(val) = &cg.no_subdir {
                cc.no_subdir = *val;
            }
            cc.numa_id = cg.memcg_numa_id.clone();
            set_fields!(cg, cc.config, [
                memcg_disabled => disabled,
                memcg_swap => swap,
                memcg_swappiness_max => swappiness_max,
                memcg_period_secs => period_secs,
                memcg_period_psi_percent_limit => period_psi_percent_limit,
                memcg_eviction_psi_percent_limit => eviction_psi_percent_limit,
                memcg_eviction_run_aging_count_min => eviction_run_aging_count_min
            ]);

            let ccs = config
                .cgroups
                .entry(cg.memcg_path.clone())
                .or_insert_with(|| Vec::new());
            ccs.push(cc);
        }

        config
    }
}

#[derive(Debug, Default)]
pub struct PathNuma {
    path: String,
    numa: Vec<u32>,
}

impl FromStr for PathNuma {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut pn = PathNuma::default();

        for pair in s.split(',') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() != 2 {
                return Err(anyhow!("{} is invalid", pair));
            }
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "path" => pn.path = value.to_string(),
                "numa-id" => {
                    pn.numa = value
                        .split(':')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.parse::<u32>())
                        .collect::<Result<Vec<u32>, _>>()?;
                }
                _ => return Err(anyhow!("{} is invalid", key)),
            }
        }

        if pn.path.is_empty() {
            return Err(anyhow!("path is required"));
        }

        Ok(pn)
    }
}

impl PathNuma {
    fn to_rpc_path_numa(&self) -> rpc::PathNuma {
        rpc::PathNuma {
            path: self.path.clone(),
            numa: self.numa.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct MemcgSetOption {
    #[structopt(long)]
    memcg_disabled: Option<bool>,
    #[structopt(long)]
    memcg_swap: Option<bool>,
    #[structopt(long)]
    memcg_swappiness_max: Option<u8>,
    #[structopt(long)]
    memcg_period_secs: Option<u64>,
    #[structopt(long)]
    memcg_period_psi_percent_limit: Option<u8>,
    #[structopt(long)]
    memcg_eviction_psi_percent_limit: Option<u8>,
    #[structopt(long)]
    memcg_eviction_run_aging_count_min: Option<u64>,
    #[structopt(long)]
    memcg_add: Vec<CgroupMemcgSetOption>,
    #[structopt(long)]
    memcg_set: Vec<CgroupMemcgSetOption>,
    #[structopt(long)]
    memcg_del: Vec<PathNuma>,
}

impl MemcgSetOption {
    pub fn to_rpc_memcg_config(&self) -> rpc::MemcgConfig {
        let mut config = rpc::MemcgConfig {
            ..Default::default()
        };

        config.default = Some(rpc::MemcgSingleConfig {
            disabled: self.memcg_disabled,
            swap: self.memcg_swap,
            swappiness_max: self.memcg_swappiness_max.map(|v| v as u32),
            period_secs: self.memcg_period_secs,
            period_psi_percent_limit: self.memcg_period_psi_percent_limit.map(|v| v as u32),
            eviction_psi_percent_limit: self.memcg_eviction_psi_percent_limit.map(|v| v as u32),
            eviction_run_aging_count_min: self.memcg_eviction_run_aging_count_min,
            ..Default::default()
        })
        .into();

        for pn in &self.memcg_del {
            config.del.push(pn.to_rpc_path_numa());
        }

        for cg in &self.memcg_add {
            config.add.push(cg.to_rpc_memcg_config_item());
        }

        for cg in &self.memcg_set {
            config.set.push(cg.to_rpc_memcg_config_item());
        }

        config
    }
}

#[derive(Debug, StructOpt)]
pub struct CompactSetOption {
    #[structopt(long)]
    compact_disabled: Option<bool>,
    #[structopt(long)]
    compact_period_secs: Option<u64>,
    #[structopt(long)]
    compact_period_psi_percent_limit: Option<u8>,
    #[structopt(long)]
    compact_psi_percent_limit: Option<u8>,
    #[structopt(long)]
    compact_sec_max: Option<i64>,
    #[structopt(long)]
    compact_order: Option<u8>,
    #[structopt(long)]
    compact_threshold: Option<u64>,
    #[structopt(long)]
    compact_force_times: Option<u64>,
}

impl CompactSetOption {
    #[allow(dead_code)]
    pub fn to_rpc_compact_config(&self) -> rpc::CompactConfig {
        let config = rpc::CompactConfig {
            disabled: self.compact_disabled,
            period_secs: self.compact_period_secs,
            period_psi_percent_limit: self.compact_period_psi_percent_limit.map(|v| v as u32),
            compact_psi_percent_limit: self.compact_psi_percent_limit.map(|v| v as u32),
            compact_sec_max: self.compact_sec_max,
            compact_order: self.compact_order.map(|v| v as u32),
            compact_threshold: self.compact_threshold,
            compact_force_times: self.compact_force_times,
            ..Default::default()
        };

        config
    }

    #[allow(dead_code)]
    pub fn to_mem_agent_compact_config(&self) -> mem_agent_lib::compact::Config {
        let mut config = mem_agent_lib::compact::Config {
            ..Default::default()
        };

        if let Some(v) = self.compact_disabled {
            config.disabled = v;
        }
        if let Some(v) = self.compact_period_secs {
            config.period_secs = v;
        }
        if let Some(v) = self.compact_period_psi_percent_limit {
            config.period_psi_percent_limit = v;
        }
        if let Some(v) = self.compact_psi_percent_limit {
            config.compact_psi_percent_limit = v;
        }
        if let Some(v) = self.compact_sec_max {
            config.compact_sec_max = v;
        }
        if let Some(v) = self.compact_order {
            config.compact_order = v;
        }
        if let Some(v) = self.compact_threshold {
            config.compact_threshold = v;
        }
        if let Some(v) = self.compact_force_times {
            config.compact_force_times = v;
        }

        config
    }
}
