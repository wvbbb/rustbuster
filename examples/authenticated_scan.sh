#!/bin/bash
# Authenticated scanning example

TARGET="http://example.com/admin"
COOKIE="session=abc123; token=xyz789"
AUTH_HEADER="Authorization: Bearer your_token_here"

echo "Running authenticated scan..."

rustbuster dir \
  -u "$TARGET" \
  -w wordlist.txt \
  -c "$COOKIE" \
  -H "$AUTH_HEADER" \
  -t 20 \
  -s 200,301,302 \
  -o authenticated_results.txt

echo "Authenticated scan complete!"
