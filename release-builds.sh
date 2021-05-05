#!/bin/bash
build () {
    if [[ $2 = "~" ]]
    then
        sed -i "s/uid: .*/uid: \~/" happ.yaml
        FILE="elemental-chat.$1.happ"
    else
        sed -i "s/uid: .*/uid: \"$2\"/" happ.yaml
        FILE="elemental-chat.$1.$2.happ"
    fi
    hc app pack . -o $FILE
}
# get the version from the chat zome Cargo.toml
VERSION=`grep -Po '^version = "\K([^"]+)' zomes/chat/Cargo.toml | sed -e "s/[.-]/_/g"`
build $VERSION 0002
build $VERSION 0001
build $VERSION 0000
build $VERSION develop
build $VERSION "~"
