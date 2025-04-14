# Introduction
**mem-agent** is a software written in Rust, designed for managing memory and reducing memory usage in Linux environments.<br>
It can be executed as a standalone executable file or integrated as a rust library into other Rust projects.<br>
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
Make sure current Linux kernel open PSI and mgLRU option.
```
CONFIG_PSI=y
CONFIG_LRU_GEN=y
```
To build mem-agent, make sure Rust is installed.
```
cd mem-agent
make
```
Run mem-agent, and it will automatically start detecting system status and reclaiming cold memory.
```
cd mem-agent
sudo target/debug/mem-agent-srv
```

# Configurations
## Feature MemCG
### memcg_disable
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

### memcg_swap
If this feature is disabled, the mem-agent will only track and reclaim file cache pages.  If this feature is enabled, the mem-agent will handle both file cache pages and anonymous pages.<br>
Default to false.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-swap true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-disabled true
```

### memcg_swappiness_max
The usage of this value is similar to the swappiness in the Linux kernel, applying a ratio of swappiness_max/200 when utilized.<br>
At the beginning of the eviction memory process for a cgroup in each run period, the coldest anonymous pages are assigned a maximum eviction value based on swappiness_max/200.<br>
When the run_eviction function of MgLRU is actually called, if the comparison ratio between the current coldest anonymous pages and file cache pages exceeds this value, then this value will be used as the swappiness.<br>
Default to 50.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-disabled true
```

### memcg_period_secs
Control the mem-agent memcg function wait period seconds.<br>
Default to 600.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-disabled true
```

### memcg_period_psi_percent_limit
Control the mem-agent memcg wait period PSI percent limit.<br>
If the percentage of memory and IO PSI stall time within the memcg waiting period for a cgroup exceeds this value, then the memcg run period for this cgroup will not be executed after this waiting period.<br>
Default to 1

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentMemcgSet json://{"period_psi_percent_limit":1}'
```

### memcg_eviction_psi_percent_limit
Control the mem-agent memcg eviction PSI percent limit.<br>
If the percentage of memory and IO PSI stall time for a cgroup exceeds this value during an eviction cycle, the eviction for this cgroup will immediately stop and will not resume until the next memcg waiting period.<br>
Default to 1.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentMemcgSet json://{"eviction_psi_percent_limit":1}'
```

### memcg_eviction_run_aging_count_min
Control the mem-agent memcg eviction run aging count min.<br>
A cgroup will only perform eviction when the number of aging cycles in memcg is greater than or equal to memcg_eviction_run_aging_count_min.<br>
Default to 3.

Set this configuration when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentMemcgSet json://{"eviction_run_aging_count_min":3}'
```

## Feature compact
### compact_disable
Control the mem-agent compact function disable or enable.<br>
Default to false.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"disabled":false}'
```

### compact_period_secs
Control the mem-agent compaction function wait period seconds.<br>
Default to 600.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"period_secs":600}'
```

### compact_period_psi_percent_limit
Control the mem-agent compaction function wait period PSI percent limit.<br>
If the percentage of memory and IO PSI stall time within the compaction waiting period exceeds this value, then the compaction will not be executed after this waiting period.<br>
Default to 1.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

### compact_psi_percent_limit
Control the mem-agent compaction function compact PSI percent limit.<br>
During compaction, the percentage of memory and IO PSI stall time is checked every second. If this percentage exceeds compact_psi_percent_limit, the compaction process will stop.<br>
Default to 5

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"compact_psi_percent_limit":5}'
```

### compact_sec_max
Control the maximum number of seconds for each compaction of mem-agent compact function.<br>
If compaction seconds is bigger than compact_sec_max during compact run period, stop compaction at once.

Default to 180.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"compact_sec_max":180}'
```

### compact_order
compact_order is use with compact_threshold.<br>
compact_order parameter determines the size of contiguous pages that the mem-agent's compaction functionality aims to achieve.<br>
For example, If the goal is to have more free pages of order 9 in the system to ensure a higher likelihood of obtaining transparent huge pages during memory allocation, then setting compact_order to 9 would be appropriate.
Default to 9.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"compact_order":9}'
```

### compact_threshold
Control the mem-agent compaction function compact threshold.<br>
compact_threshold is the pages number.<br>
When examining the /proc/pagetypeinfo, if there's an increase in the number of movable pages of orders smaller than the compact_order compared to the amount following the previous compaction period, and this increase surpasses a certain threshold specifically, more than compact_threshold number of pages, or the number of free pages has decreased by compact_threshold since the previous compaction. Current compact run period will not do compaction because there is no enough fragmented pages to be compaction.<br>
This design aims to minimize the impact of unnecessary compaction calls on system performance.<br>
Default to 1024.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"compact_threshold":1024}'
```

### compact_force_times
Control the mem-agent compaction function force compact times.<br>
After one compaction during a run period, if there are consecutive instances of compact_force_times run periods where no compaction occurs, a compaction will be forced regardless of the system's memory state.<br>
If compact_force_times is set to 0, will do force compaction each period.<br>
If compact_force_times is set to 18446744073709551615, will never do force compaction.<br>
Default to 18446744073709551615.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
$ PODID="12345"
$ kata-agent-ctl connect --server-address "unix:///var/run/kata/$PODID/root/kata.hvsock" --hybrid-vsock \
--cmd 'MemAgentCompactSet json://{"compact_force_times":18446744073709551615}'
```
