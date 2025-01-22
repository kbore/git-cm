#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

# 脚本执行期间去除所有本地化设置，保证不同环境脚本执行结果一致性
export LC_ALL=POSIX

function git_config() {
    git config --local include.path ../.gitconfig
}

function build_target() {
    # released binary require special version of glibc
    cargo zigbuild --target x86_64-unknown-linux-gnu.2.17 --release
}

git_config
build_target