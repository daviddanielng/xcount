#!/bin/bash

#cargo run -- --username shs,jss,jsj
#cargo run -- --input ./ttt
cargo run -- --input ./ttt -f json -o ./outputs
#cargo run -- --set-exe-path /usr/bin/google-chrome