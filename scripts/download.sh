#!/bin/bash
cd "$(git rev-parse --show-toplevel)"
for i in $(seq 1 106)
do
  echo ${i}
  curl "https://poses.live/problems/${i}/download" > "problems/${i}.problem"
done
