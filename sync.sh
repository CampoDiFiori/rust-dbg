#!/bin/bash

PROJECT_DIR="$HOME/cool"
REMOTE_DIR="/root/cool"

fswatch -o "$PROJECT_DIR" | while read f
do
    if git -C "$PROJECT_DIR" ls-files --error-unmatch "$f" > /dev/null 2>&1; then
        rsync -avz --delete "$PROJECT_DIR/" vm:"$REMOTE_DIR"
        echo "Synced changes to VM"
    fi
done
