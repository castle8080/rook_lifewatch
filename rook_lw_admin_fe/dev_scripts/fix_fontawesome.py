#!/usr/bin/env python3
#
# This project is using fontawesome with bulma.
# The fontawesome css has a different default path to fonts than,
# what the app is doing. This script edits the css to conform to
# how the app expects deployment of font files.
#
import os
import re

static_dir = "static"

# Find any fontawesome css.
def find_css_files():
    css_files = []
    for d_ent in os.listdir(static_dir):
        fa_dir = os.path.join(static_dir, d_ent)
        if d_ent.startswith("fontawesome") and os.path.isdir(fa_dir):
            css_dir = os.path.join(fa_dir, "css")
            if os.path.isdir(css_dir):
                for f_ent in os.listdir(css_dir):
                    if f_ent.lower().endswith(".css"):
                        css_files.append(os.path.join(css_dir, f_ent))
    return css_files

def fix_css_content(css_content):
    semi_colo_re = re.compile(r';')
    web_fonts_re = re.compile(r'../webfonts/')

    # Splitting isn't needed but makes checking logic easier.
    parts = []
    for css_content_part in semi_colo_re.split(css_content):
        new_content = web_fonts_re.sub("webfonts/", css_content_part)
        if new_content != css_content_part:
            print(f"----------------------------------------")
            print(f"old: {css_content_part}")
            print(f"new: {new_content}")
        parts.append(new_content)

    return ";".join(parts)

def run_css_file_fixes():
    css_files = find_css_files()
    for css_file in css_files:
        with open(css_file, 'r') as fh:
            css_content = fh.read()

        new_css_content = fix_css_content(css_content)
        if css_content == new_css_content:
            print("No changes need to be saved.")
        else:
            print(f"Writing changes found in file: {css_file}")
            with open(css_file, 'w') as fh:
                fh.write(new_css_content)

run_css_file_fixes()
