#!/bin/bash
cd "$(git rev-parse --show-toplevel)"
for i in $(seq 60 78)
do
  echo ${i}
  curl "https://poses.live/problems/${i}/download" > "problems/${i}.problem"
done
