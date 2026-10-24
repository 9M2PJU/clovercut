#!/usr/bin/env python3
"""Select existing desktop build recipes; releases default to every recipe."""
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--target', default='all')
parser.add_argument('--github-output', type=Path)
args = parser.parse_args()
recipes = json.loads(Path(__file__).with_name('desktop-targets.json').read_text())
known = {f"{row['platform']}-{row['arch']}" for rows in recipes.values() for row in rows}
if args.target != 'all' and args.target not in known:
    parser.error(f'Unknown desktop target: {args.target}')
output = []
for job, rows in recipes.items():
    chosen = [row for row in rows if args.target == 'all' or f"{row['platform']}-{row['arch']}" == args.target]
    # Keep syntactically valid recipes even on skipped jobs; the run flag is
    # evaluated before matrix expansion and prevents those jobs from starting.
    output.append(f'{job}_matrix=' + json.dumps({'include': chosen or rows}, separators=(',', ':')))
    output.append(f'run_{job}=' + str(bool(chosen)).lower())
text = '\n'.join(output) + '\n'
if args.github_output:
    with args.github_output.open('a') as target:
        target.write(text)
else:
    print(text, end='')
