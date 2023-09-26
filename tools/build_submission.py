#!/usr/bin/python3

import argparse
from collections import deque
import re
import sys

parser = argparse.ArgumentParser()
parser.add_argument("-o", "--output", required=True, help="output file")
args = parser.parse_args()

skip_line_re = re.compile(r".*// submission::skip\n")
mod_line_re = re.compile(r"((pub )?)mod ([a-z_]*);\n")


def process(file_name, output_file):
    lines = []

    with open(f"src/{file_name}") as f:
        for line in f:
            if skip_line_re.fullmatch(line) is not None:
                continue
            m = mod_line_re.fullmatch(line)
            if m is not None:
                prefix = m.group(1)
                mod = m.group(3)
                process_mod(prefix, mod, mod + ".rs", output_file)
            else:
                output_file.write(line)


def process_mod(prefix, mod_name, file_name, output_file):
    output_file.write(f"{prefix}mod {mod_name} {{\n")
    process(file_name, output_file)
    output_file.write(f"}} // mod {mod_name}\n\n")


with open(args.output, "w") as output_file:
    process("lib.rs", output_file)
    process("main.rs", output_file)
