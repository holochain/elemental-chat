#!/bin/bash

# How to?
# https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token
# git config --global github .token YOUR_TOKEN

version=$1
text=$2
branch=$(git rev-parse --abbrev-ref HEAD)
token=$(git config --global github.token)
USER="holochain"
REPO="elemental-chat"
generate_post_data()
{
  cat <<EOF
{
  "tag_name": "$version",
  "target_commitish": "$branch",
  "name": "v$version",
  "body": "$text",
  "draft": true,
  "prerelease": false
}
EOF
}

echo "Create release $version for repo: $USER/$REPO branch: $branch"

payload=$(
  jq --null-input \
     --arg tag "$version" \
     --arg name "v$version" \
     --arg body "$text" \
     '{ tag_name: $tag, name: $name, body: $body, draft: true }'
)

response=$(
  curl --fail \
       --netrc \
       --silent \
       --location \
       --data "$(generate_post_data)" \
       "https://api.github.com/repos/${USER}/${REPO}/releases?access_token=$token"
)

upload_url="$(echo "$response" | jq -r .upload_url | sed -e "s/{?name,label}//")"
version=$(echo $1 | tr .- _)
happ_file="elemental-chat.$version.happ"
curl --netrc \
     -H "Authorization: token $token" \
     -H "Content-Type: $(file -b --mime-type $happ_file)" \
     --data-binary "@$happ_file" \
     "$upload_url?name=$(basename "$happ_file")"
dna_file="elemental-chat.$version.dna"
curl --netrc \
    -H "Authorization: token $token" \
    -H "Content-Type: $(file -b --mime-type $dna_file)" \
    --data-binary "@$dna_file" \
    "$upload_url?name=$(basename "$dna_file")"
