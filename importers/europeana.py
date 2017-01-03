"""This script imports all (public domain) news papers from the Europeana
project into a dictionary for further processing."""

import os
import re
import sys
import zipfile

BASE_URL="http://data.theeuropeanlibrary.org/download/newspapers-by-country/"

# import common module from same directory as this script is run from
sys.path.append(os.path.abspath(sys.argv[0]))
import common


def main(base_url, language, target):
    """Retrieve all zip files for the specified language and save them into
    `target`."""
    is_language = re.compile(r'^[A-Z]{3}/?$')
    # a language is identified by three letters in upper case.
    langs = [l.rstrip('/') for l in common.get_links(base_url) if is_language.search(l)]
    try:
        index = [l.lower() for l in langs].index(language.lower())
    except ValueError:
        print('Language', language, 'not found, the following languages exist:',
                ' '.join(langs))
        sys.exit(3)

    if not os.path.exists(target):
        os.makedirs(target)

    lang_url = '/'.join((base_url, langs[index]))
    for download in common.get_links(lang_url):
        if not download.lower().endswith('.zip'):
            continue # ignore non-zip files
        src_url = '/'.join((lang_url, download))
        target_zip = os.path.join(target, download)
        print("Downloading",target_zip)
        common.download_to(src_url, target_zip)
        # unzip file, remove it
        # \r is necessary to overwrite progress bar
        print("\rExtracting",target_zip)
        zipfile.ZipFile(target_zip).extractall(target)
        os.remove(target_zip)

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage:",sys.argv[0],"<language> <target_directory>")
        print(("<language> has to be an ISO-639 three-letter language code as "
            "e.g. \"fra\"; case doesn't matter."))
        sys.exit(2)
    else:
        main(BASE_URL, sys.argv[1], sys.argv[2])

