#!/bin/bash

if [ -z "$1" ]; then
    echo "Error: No wallpaper path provided" >&2
    echo "Usage: $0 <wallpaper-path>" >&2
    exit 1
fi

WALLPAPER_PATH="$1"
CONFIG_FILE="$HOME/.config/river-swww/config.json"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Config file not found at $CONFIG_FILE" >&2
    exit 1
fi

# Get current tags from river-bedload
TAGS_JSON=$(river-bedload -print active)
if [ $? -ne 0 ]; then
    echo "Error: Failed to get tags from river-bedload" >&2
    exit 1
fi

# Find the tag where active, focused, and occupied are all true (take first match)
CURRENT_TAG_ID=$(echo "$TAGS_JSON" | jq -r 'first(.[] | select(.active == true and .focused == true and .occupied == true) | .id) | tostring')
if [ -z "$CURRENT_TAG_ID" ]; then
    echo "Error: No tag found that is simultaneously active, focused, and occupied" >&2
    exit 1
fi

# Update the config file
jq --arg tag_id "$CURRENT_TAG_ID" --arg wallpaper "$WALLPAPER_PATH" '.tags[$tag_id] = $wallpaper' "$CONFIG_FILE" > "$CONFIG_FILE.tmp"

if [ $? -ne 0 ]; then
    echo "Error: Failed to update config file" >&2
    rm -f "$CONFIG_FILE.tmp"
    exit 1
fi

# Replace the original config file
mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
