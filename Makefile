# Copyright (C) 2024 Ant group. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0

.PHONY: default static clean sudo_test sudo_clean

default:
	cargo build --workspace

static:
	cargo build --workspace --target x86_64-unknown-linux-musl

clean:
	cargo clean

sudo_test:
	sudo -E env "PATH=$$PATH" cargo test

sudo_clean:
	sudo -E env "PATH=$$PATH" cargo clean
