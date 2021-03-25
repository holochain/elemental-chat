#!/bin/bash
build () {
    sed -i "s/uuid: .*/uuid: \"$2\"/" happ.yaml
    hc dna pack . -o elemental-chat.$1.$2.happ
}
#build 0_1_0_alpha1 0002
#build 0_1_0_alpha1 0001
build 0_1_0_alpha1 develop
