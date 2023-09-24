#!/usr/bin/python3

import argparse
from collections import deque
import re
import sys

parser = argparse.ArgumentParser()
parser.add_argument("-o", "--output", required=True, help="output file")
args = parser.parse_args()

mod_line_re = re.compile(r"mod ([a-z_]*);\n")


def process(file_name, output_file):
    lines = []

    with open(f"src/{file_name}") as f:
        for line in f:
            m = mod_line_re.fullmatch(line)
            if m is not None:
                mod = m.group(1)
                process_mod(mod, mod + ".rs", output_file)
            else:
                output_file.write(line)


def process_mod(mod_name, file_name, output_file):
    output_file.write(f"mod {mod_name} {{\n")
    process(file_name, output_file)
    output_file.write(f"}} // mod {mod_name}\n")


with open(args.output, "w") as output_file:
    process("main.rs", output_file)
    process_mod("sudoku_game", "lib.rs", output_file)
