#!/usr/bin/env bash

inotifywait -q -m -e close_write $1 |
while read -r filename event; do
  python3 $1
done
