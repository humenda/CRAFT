#!/usr/bin/env python3
"""
The DGT translation memory is a multilingual corpus of phrases. It consists of
many zip files with multiple .tmx (XML) documents. Each document contains many
phrases. This script downloads the available zip files.
"""
import os
import sys

# figure out base directory and add to path
sys.path.insert(0, os.path.dirname(os.path.abspath(sys.argv[0])))
import common

BASE_URL = "http://data.europa.eu/euodp/en/data/dataset/dgt-translation-memory"

def main(output_directory):
    if not os.path.exists(output_directory):
        os.makedirs(output_directory)
    links = list(l for l in common.get_links(BASE_URL) if l.endswith('.zip'))
    for index, link in enumerate(links):
        fname = link.split('/')[-1]
        target = os.path.join(output_directory, fname)
        print("Downloading %s/%s: %s" % (index+1, len(links), link))
        common.download_to(link, target)

if len(sys.argv) != 2:
    print("Usage: %s <target directory>\n" % sys.argv[0])
    print("This script downloads all European digital translation aids to the "
        "specified directory.")
    sys.exit(1)
else:
    main(sys.argv[1])
