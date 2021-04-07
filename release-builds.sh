#!/bin/bash
build () {
    sed -i "s/uuid: .*/uuid: \"$2\"/" happ.yaml
    hc app pack . -o elemental-chat.$1.$2.happ
}
# get the version from the chat zome Cargo.toml
VERSION=`grep -Po '^version = "\K([^"]+)' zomes/chat/Cargo.toml | sed -e "s/[.-]/_/g"`
build $VERSION 0002
build $VERSION 0001
build $VERSION develop
