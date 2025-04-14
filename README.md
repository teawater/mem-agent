# Introduction
**mem-agent** is a software written in Rust, designed for managing memory and reducing memory usage in Linux environments.<br>
It can be executed as a standalone executable file or integrated as a rust library into other Rust projects.<br>
<br>
The mem-agent has been integrated into the kata-agent as a library.
https://github.com/kata-containers/kata-containers/blob/main/docs/how-to/how-to-use-memory-agent.md

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
# Feature and confguration
## Feature PSI
During memory reclamation and compaction, mem-agent monitors system pressure using Pressure Stall Information (PSI).<br>
If the system pressure becomes too high, memory reclamation or compaction will automatically stop.

This feature helps the mem-agent reduce its overhead on system performance.

**The configuration of PSI functions is integrated with other features.**

## Feature MemCG
Use the Linux kernel MgLRU feature to monitor each cgroup's memory usage and periodically reclaim cold memory.

During each run period, memcg calls the run_aging function of MgLRU for each cgroup to mark the hot and cold states of the pages within it.<br>
Then, it calls the run_eviction function of MgLRU for each cgroup to reclaim a portion of the cold pages that have not been accessed for three periods.

After the run period, the memcg will enter a sleep period. Once the sleep period is over, it will transition into the next run period, and this cycle will continue.

**The following are the configurations of the Feature MemCG:**

### memcg_disable
Control the mem-agent memcg function disable or enable.<br>
Default to false.

Set memcg_disable when start mem-agent-srv:
```bash
sudo target/debug/mem-agent-srv --memcg-disabled true
```

For a running mem-agent-srv, this configuration can be dynamically modified using the mem-agent-ctl command.
```bash
sudo target/debug/mem-agent-ctl memcgset --memcg-disabled true
```

