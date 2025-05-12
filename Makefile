# Copyright (C) 2024 Ant group. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0

.PHONY: default clean

default:
	cargo build --workspace

clean:
	cargo clean

test:
	sudo -E env "PATH=$$PATH" cargo test
