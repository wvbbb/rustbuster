#!/bin/bash
# Basic directory enumeration example

echo "Running basic directory scan..."
rustbuster dir \
  -u http://testphp.vulnweb.com \
  -w wordlist.txt \
  -t 10 \
  -s 200,301,302,403 \
  -o results.txt

echo "Scan complete! Results saved to results.txt"
