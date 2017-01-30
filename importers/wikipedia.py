#!/usr/bin/env python3
# This file downloads a wikipedia dump for a given language. See usage for more
# information.
import os
import sys
import urllib.request

USAGE = """%s <language> <output directory>
Download the latest wikipedia dump (latest version of each article only) to the
specified output directory. <language> has to be a two-letter code, matching
those used by the subdomains from Wikipedia (e.g. fr.wikipedia.org for the
French one).""" % sys.argv[0]

DOWNLOAD_LOCATION = "https://dumps.wikimedia.org/{0}wiki/latest/{0}wiki-latest-pages-articles.xml.bz2"


def print_percentage(percentage):
    """Print percentage of download to a terminal."""
    sys.stdout.write('\r' + ' ' * 40 + '\r') # user might have typed something
    sys.stdout.write(('%.1f %%' % percentage).rjust(7))
    sys.stdout.flush()


def main():
    if len(sys.argv) != 3:
        print(USAGE)
        sys.exit(1)

    language, output_directory = sys.argv[1:3]
    if len(language) != 2:
        print("The language code must be a two-letter identifier, as e.g. de for German")
        sys.exit(2)

    if not os.path.exists(output_directory):
        os.makedirs(output_directory)

    url = DOWNLOAD_LOCATION.format(language)
    output_file = os.path.join(output_directory, url.split('/')[-1])

    # similar to common.download_to, but overwrites _always_
    with urllib.request.urlopen(url) as u:
        size = int(u.info().get('Content-Length'))
        print('Downloading %.1f MB from %s' % (size / 1024 / 1024, url))
        downloaded_so_far = 0
        fetch_at_once = 150000
        with open(output_file, 'wb') as f:
            read = u.read(fetch_at_once)
            while read:
                f.write(read)
                downloaded_so_far += len(read)
                print_percentage(downloaded_so_far / (size / 100))
                read = u.read(fetch_at_once)
        print()

main()
