#!/bin/bash
# Advanced scanning with multiple features

TARGET="http://example.com"
WORDLIST="/usr/share/seclists/Discovery/Web-Content/common.txt"

echo "Starting advanced scan on $TARGET"

# Directory scan with extensions and recursive mode
rustbuster dir \
  -u "$TARGET" \
  -w "$WORDLIST" \
  -x php,html,js,txt \
  -R \
  --depth 2 \
  -t 30 \
  --timeout 15 \
  -s 200,301,302,403 \
  -n 404,500 \
  --backup-extensions \
  -o results.json \
  --output-format json \
  -v

echo "Scan complete!"
