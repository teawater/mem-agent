// Copyright (C) 2025 Kylin Soft. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use nix::sys::statfs::{statfs, CGROUP2_SUPER_MAGIC};
use std::path::Path;

pub const CGROUP_PATH: &str = "/sys/fs/cgroup/";
pub const MEMCGS_V1_PATH: &str = "/sys/fs/cgroup/memory";

pub fn is_cgroup_v2() -> Result<bool> {
    let cgroup_path = Path::new("/sys/fs/cgroup");

    let stat = statfs(cgroup_path).map_err(|e| anyhow!("statfs {:?} failed: {}", cgroup_path, e))?;
	Ok(stat.filesystem_type() == CGROUP2_SUPER_MAGIC)
}
