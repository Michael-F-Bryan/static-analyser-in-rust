#!/bin/env python3

"""
Find all Rust code in markdown files and run `rustfmt` over them.

USAGE:
    ./rustfmt.py [<root_dir>]
"""

from pathlib import Path
import subprocess
import json
import os
import docopt


def detect_rust(src):
    """
    Find which bits of this Markdown file contain Rust code.
    """
    lines = []
    in_code_block = False
    start_of_code_block = 0

    for i, line in enumerate(src.splitlines()):
        if '```rust' in line:
            start_of_code_block = i
            in_code_block = True
        elif '```' in line and in_code_block:
            lines.append((start_of_code_block + 1, i - 1))
            in_code_block = False

    return lines


def analyse(filename):
    src = filename.read_text()

    ranges = []

    for span in detect_rust(src):
        payload = {
            'file': str(filename),
            'range': span,
        }
        ranges.append(payload)

    return ranges


def run_rustfmt(inputs):
    jason = json.dumps(inputs)
    output = subprocess.run(['rustfmt', '--file-lines', jason],
                            check=True,
                            stdin=subprocess.DEVNULL,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE)

    if output.returncode != 0:
        raise 'Return code was non-zero'

    print(output.stdout)
    print()
    print(output.stderr)


def main(root):
    markdown_files = [Path(path) / file for (path, _, files)
                      in os.walk(root) for file in files]
    ranges = []

    for path in markdown_files:
        if path.suffix == '.md':
            ranges.extend(analyse(path))

    run_rustfmt(ranges)


if __name__ == '__main__':
    args = docopt.docopt(__doc__)
    main(args.get('<root_dir>') or '.')
