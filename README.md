# Introduction
**mem-agent** is a software written in Rust, designed for managing memory and reducing memory usage in Linux environments.<br>
It can be executed as a standalone executable file or integrated as a rust library into other Rust projects.<br>
The mem-agent offers a wide range of configurable options that can be tailored as needed, with most of them supporting dynamic configuration.<br>
<br>
The mem-agent has been integrated into the kata-agent as a library.
https://github.com/kata-containers/kata-containers/blob/main/docs/how-to/how-to-use-memory-agent.md

The following are the features of the mem-agent:

## Feature MemCG
Use the Linux kernel MgLRU feature to monitor each cgroup's memory usage and periodically reclaim cold memory.

During each run period, memcg calls the run_aging function of MgLRU for each cgroup to mark the hot and cold states of the pages within it.<br>
Then, it calls the run_eviction function of MgLRU for each cgroup to reclaim a portion of the cold pages that have not been accessed for three periods.

After the run period, the memcg will enter a sleep period. Once the sleep period is over, it will transition into the next run period, and this cycle will continue.

## Feature compact
In numerous scenarios, the host requires more contiguous free pages while not wanting system performance to be impacted by frequent page defragmentation.<br>
For example, when aiming to acquire more Transparent HugePages (THP) without enabling the defragmentation option that affects the speed of memory allocation and system performance, this relies on there being some contiguous free pages in the system.<br>
Furthermore, inside a Kata container, the VM balloon free page reporting feature utilized by reclaim_guest_freed_memory necessitates at least a contiguous block of order 10 pages (a page block) to be released from the host.<br>
The Feature compact is specifically designed to address these issues.

During each run period, compact check the continuity of free pages within the system. If necessary, the compact will invoke the Linux compaction feature to reorganize fragmented pages.<br>
After the run period, the compact will enter a sleep period. Once the sleep period is over, it will transition into the next run period, and this cycle will continue.

## Feature PSI
During memory reclamation and compaction, mem-agent monitors system pressure using Pressure Stall Information (PSI).<br>
If the system pressure becomes too high, memory reclamation or compaction will automatically stop.

This feature helps the mem-agent reduce its overhead on system performance.

# Quick start
## Linux kernel
Make sure current Linux kernel open PSI and mgLRU option.
```
CONFIG_PSI=y
CONFIG_LRU_GEN=y
```
For openEuler using cgroup v2, also ensure the following option is enabled:
```
CONFIG_PSI_CGROUP_V1=y
```
## Build mem-agent
To build mem-agent, make sure Rust 1.75 or newer version is installed (https://www.rust-lang.org/tools/install).
```
cd mem-agent
make
```
## Open PSI in grub
### Update /etc/default/grub
Add "psi=1" to /etc/default/grub.
```
sudo sed -i '/^GRUB_CMDLINE_LINUX=/ { /psi=1/! s/"$/ psi=1"/ }' /etc/default/grub
```

For openEuler using cgroup v2, add "psi=1 psi_v1=1" to /etc/default/grub.
```
sudo sed -i '/^GRUB_CMDLINE_LINUX=/ { /psi=1/!s/"$/ psi=1"/; /psi_v1=1/!s/"$/ psi_v1=1"/; }' /etc/default/grub
```
### Update grub config
For the system that has "update-grub".
```
sudo update-grub
```

For the others.
```
sudo grub2-mkconfig -o /boot/grub2/grub.cfg
```

Reboot after update grub config.

## Run mem-agent
Run mem-agent, and it will automatically start detecting system status and reclaiming cold memory.
```
cd mem-agent
sudo target/debug/mem-agent-srv
```

# Configurations
## config log
### set the log file
Set the log file.
If not set, the log will be output to the current terminal.

Set the log to file /var/log/mem_agent.log.
```bash
sudo target/debug/mem-agent-srv --log-file /var/log/mem_agent.log
```
### set the log level
Set the log level to trace, debug, info, warn, error, or critical.
Default to trace.

Set the log level to debug.
```bash
sudo target/debug/mem-agent-srv --log-level debug
```
For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl loglevelset debug
```

## Feature MemCG
### Base configuration
For memory cgroups that are not individually configured with the --memcg-cgroups parameter (as detailed below), their memory reclamation will be governed by the following configurations.
#### memcg_disable
Control the mem-agent memcg function disable or enable.<br>
Default to false.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-disabled true
```

#### memcg_swap
If this feature is disabled, the mem-agent will only track and reclaim file cache pages.  If this feature is enabled, the mem-agent will handle both file cache pages and anonymous pages.<br>
Default to false.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-swap true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-swap true
```

#### memcg_swappiness_max
The usage of this value is similar to the swappiness in the Linux kernel, applying a ratio of swappiness_max/200 when utilized.<br>
At the beginning of the eviction memory process for a cgroup in each run period, the coldest anonymous pages are assigned a maximum eviction value based on swappiness_max/200.<br>
When the run_eviction function of MgLRU is actually called, if the comparison ratio between the current coldest anonymous pages and file cache pages exceeds this value, then this value will be used as the swappiness.<br>
Default to 50.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-swappiness-max 50
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-swappiness-max 50
```

#### memcg_period_secs
Control the mem-agent memcg function wait period seconds.<br>
Default to 600.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-period-secs 600
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-period-secs 600
```

#### memcg_period_psi_percent_limit
Control the mem-agent memcg wait period PSI percent limit.<br>
Execution of the memcg run period for a cgroup will resume ​only when the aggregate percentage of memory or IO PSI (use the bigger one) stall time across all its accumulated pending waiting periods falls below this threshold.​​<br>
Default to 1

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-period-psi-percent-limit 1
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-period-psi-percent-limit 1
```

#### memcg_eviction_psi_percent_limit
Control the mem-agent memcg eviction PSI percent limit.<br>
If the percentage of memory or IO PSI (use the bigger one) stall time for a cgroup exceeds this value during an eviction cycle, the eviction for this cgroup will immediately stop and will not resume until the next memcg waiting period.<br>
Default to 1.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-eviction-psi-percent-limit 1
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-eviction-psi-percent-limit 1
```

#### memcg_eviction_run_aging_count_min
Control the mem-agent memcg eviction run aging count min.<br>
A cgroup will only perform eviction when the number of aging cycles in memcg is greater than or equal to memcg_eviction_run_aging_count_min.<br>
Default to 3.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-eviction-run-aging-count-min 3
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-eviction-run-aging-count-min 3
```

### configuration for special memory cgroups and NUMA
If you need to configure specific memory cgroups and NUMA with custom settings rather than using default configurations, you can utilize the following configuration.

#### Set configuration as the option of mem-agent-srv
The following command starts the mem-agent server with three custom memory cgroup configurations:
1. For the memory cgroup /system.slice/ModemManager.service and its subdirectories NUMA nodes 1 and 2 only (explicitly excluding other nodes):
* Sets period-secs = 300
* All other configurations retain default values
2. For the memory cgroup /system.slice/snapd.socket (excluding its subdirectories):
* Sets period-psi-percent-limit = 10
* All other configurations retain default values
3. For /system.slice/bolt.service and its subdirectories:
* All memcg features are disabled

All other memory cgroups and NUMA nodes across the system will use the base configurations.
```bash
sudo target/debug/mem-agent-srv --memcg-cgroups path=/system.slice/ModemManager.service,numa-id=1:2,period-secs=300 path=/system.slice/snapd.socket,no-subdir=true,period-psi-percent-limit=10 path=/system.slice/bolt.service,disabled=true
```
following is the sub-configurations of --memcg-cgroups:
* path: The path of the memory cgroup to be configured.
* nuima-id: The NUMA node ids ( separated by : ) to be configured.
* no-subdir: If true, the sub-directory of path will not be configured.
* disabled: If true, All memcg features are disabled for this path.
* swap: Same with the base configuration --memcg-swap.
* swappiness-max: Same with the base configuration --memcg-swappiness-max.
* period-secs: Same with the base configuration --memcg-period-secs.
* period-psi-percent-limit: Same with the base configuration --memcg-period-psi-percent-limit.
* eviction-psi-percent-limit: Same with the base configuration --memcg-eviction-psi-percent-limit.
* eviction-run-aging-count-min: Same with the base configuration --memcg-eviction-run-aging-count-min.

#### Set configuration as the option of mem-agent-ctl
##### Add
Add special configuration for some memory cgroups.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-add path=/system.slice/ModemManager.service,numa-id=1:2,period-secs=300 path=/system.slice/snapd.socket,no-subdir=true,period-psi-percent-limit=10 path=/system.slice/bolt.service,disabled=true
```
The sub-configurations of --memcg-add are same with --memcg-cgroups of mem-agent-srv.
##### Delete
Delete special configuration for some memory cgroups.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-del path=/system.slice/ModemManager.service,numa-id=1:2 path=/system.slice/snapd.socket path=/system.slice/bolt.service
```
following is the sub-configurations of --memcg-del:
* path: The path of the memory cgroup to be configured.
* nuima-id: The NUMA node ids ( separated by : ) to be configured.

##### Update
Update special configuration for some memory cgroups.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-set path=/system.slice/ModemManager.service,numa-id=1:2,disabled=true path=/system.slice/snapd.socket,period-secs=300 path=/system.slice/bolt.service,disabled=false
```
The sub-configurations of --memcg-set are same with --memcg-cgroups of mem-agent-srv.

## Feature compact
### compact_disable
Control the mem-agent compact function disable or enable.<br>
Default to false.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-disabled true
```

### compact_period_secs
Control the mem-agent compaction function wait period seconds.<br>
Default to 600.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-period-secs 600
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-period-secs 600
```

### compact_period_psi_percent_limit
Control the mem-agent compaction function wait period PSI percent limit.<br>
Execution of the compact run period for a cgroup will resume ​only when the aggregate percentage of memory or IO PSI (use the bigger one) stall time across all its accumulated pending waiting periods falls below this threshold.​​<br>
Default to 1.

Set compact_period_psi_percent_limit when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-period-psi-percent-limit 1
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-period-psi-percent-limit 1
```

### compact_psi_percent_limit
Control the mem-agent compaction function compact PSI percent limit.<br>
During compaction, the percentage of memory or IO PSI (use the bigger one) stall time is checked every second. If this percentage exceeds compact_psi_percent_limit, the compaction process will stop.<br>
Default to 5

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-psi-percent-limit 1
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-psi-percent-limit 1
```

### compact_sec_max
Control the maximum number of seconds for each compaction of mem-agent compact function.<br>
If compaction seconds is bigger than compact_sec_max during compact run period, stop compaction at once.

Default to 300.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-sec-max 300
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-sec-max 300
```

### compact_order
compact_order is use with compact_threshold.<br>
compact_order parameter determines the size of contiguous pages that the mem-agent's compaction functionality aims to achieve.<br>
For example, If the goal is to have more free pages of order 9 in the system to ensure a higher likelihood of obtaining transparent huge pages during memory allocation, then setting compact_order to 9 would be appropriate.
Default to 9.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-order 9
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-order 9
```

### compact_threshold
Control the mem-agent compaction function compact threshold.<br>
compact_threshold is the pages number.<br>
When examining the /proc/pagetypeinfo, if there's an increase in the number of movable pages of orders smaller than the compact_order compared to the amount following the previous compaction period, and this increase surpasses a certain threshold specifically, more than compact_threshold number of pages, or the number of free pages has decreased by compact_threshold since the previous compaction. Current compact run period will not do compaction because there is no enough fragmented pages to be compaction.<br>
This design aims to minimize the impact of unnecessary compaction calls on system performance.<br>
Default to 1024.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-threshold 1024
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-threshold 1024
```

### compact_force_times
Control the mem-agent compaction function force compact times.<br>
After one compaction during a run period, if there are consecutive instances of compact_force_times run periods where no compaction occurs, a compaction will be forced regardless of the system's memory state.<br>
If compact_force_times is set to 0, will do force compaction each period.<br>
If compact_force_times is set to 18446744073709551615, will never do force compaction.<br>
Default to 18446744073709551615.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --compact-force-times 18446744073709551615
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl compactset --compact-force-times 18446744073709551615
```
