#!/usr/bin/env bash

FILE=$1

function find_module_changes() {
    echo "## Modules"
    echo "- Added"
    grep '\[+\] modules:' "$FILE" | sed 's/.*modules: /  - /' || echo "  n/a"
    echo "- Removed"
    grep '\[-\] modules:' "$FILE" | sed 's/.*modules: /  - /' || echo "  n/a"
    echo
}

function find_removals() {
    echo "## Removals"
    # Find all the modules with changes and pull in all the changes after it
    grep -n -E '\[.*\] idx: .*\((calls:.*|storage:.*)\)' "$FILE" |
    while read -r mod_line; do
        module=$(echo "$mod_line" | sed -E 's/^[0-9]+:[[:space:]]*\[([^]]+)\].*/\1/')
        mod_line_number=$(echo "$mod_line" | sed -E 's/^([0-9]+):.*/\1/')
        mod_line_number_plus=$(($mod_line_number + 1))
        # Find all the [-] lines after that line until the next empty line
        lines=$(sed -n -E "${mod_line_number_plus},\$ {
            /\[-\]/ {
                p
                a\\
                |
            }
            /^$/ q
        }" "$FILE")
        # If some were found, then echo out the header, and the lines
        if [ -n "$lines" ]; then
          echo "- $module"
          echo $lines | tr "|" "\n" |
          while read -r line; do
          if [ -n "${line}" ]; then
            echo "  - ${line}"
          fi
          done
        fi
    done || echo "  n/a"
    echo
}

function find_changes() {
    echo "## Changes"
    # Find all the modules with changes and pull in all the changes after it
    grep -n -E '\[.*\] idx: .*\((calls:.*|storage:.*)\)' "$FILE" |
    while read -r mod_line; do
        module=$(echo "$mod_line" | sed -E 's/^[0-9]+:[[:space:]]*\[([^]]+)\].*/\1/')
        mod_line_number=$(echo "$mod_line" | sed -E 's/^([0-9]+):.*/\1/')
        mod_line_number_plus=$(($mod_line_number + 1))
        # Find all the [extrinsic] lines after that line until the next empty line
        lines=$(sed -n -E "${mod_line_number_plus},\$ {
            /\[[^\+-]+\]/ {
                p
                a\\
                |
            }
            /^$/ q
        }" "$FILE")
        # If some were found, then echo out the header, and the lines
        if [ -n "$lines" ]; then
          echo "- $module"
          echo $lines | tr "|" "\n" |
          while read -r line; do
          if [ -n "${line}" ]; then
            echo "  - ${line}"
          fi
          done
        fi
    done || echo "  n/a"
    echo
}

function find_additions() {
    echo "## Additions"
    # Find all the modules with changes and pull in all the changes after it
    grep -n -E '\[.*\] idx: .*\((calls:.*|storage:.*)\)' "$FILE" |
    while read -r mod_line; do
        module=$(echo "$mod_line" | sed -E 's/^[0-9]+:[[:space:]]*\[([^]]+)\].*/\1/')
        mod_line_number=$(echo "$mod_line" | sed -E 's/^([0-9]+):.*/\1/')
        mod_line_number_plus=$(($mod_line_number + 1))
        # Find all the [+] lines after that line until the next empty line
        lines=$(sed -n -E "${mod_line_number_plus},\$ {
            /\[\+\]/ {
                p
                a\\
                |
            }
            /^$/ q
        }" "$FILE")
        # If some were found, then echo out the header, and the lines
        if [ -n "$lines" ]; then
          echo "- $module"
          echo $lines | tr "|" "\n" |
          while read -r line; do
          if [ -n "${line}" ]; then
            echo "  - ${line}"
          fi
          done
        fi
    done || echo "  n/a"
    echo
}

echo "------------------------------ SUMMARY -------------------------------"
echo "⚠️ This filter is here to help spotting changes that should be reviewed carefully."
echo "⚠️ It catches only index changes, deletions and value decreases."
echo

find_module_changes "$FILE"
find_removals "$FILE"
find_additions "$FILE"
find_changes "$FILE"
echo "----------------------------------------------------------------------"
