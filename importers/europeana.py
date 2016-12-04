"""This script imports all (public domain) news papers from the Europeana
project into a dictionary for further processing."""

from html.parser import HTMLParser
import os
import re
import sys
import urllib.request
import zipfile

BASE_URL="http://data.theeuropeanlibrary.org/download/newspapers-by-country/"

class LinkExctractor(HTMLParser):
    """Extract all URL's from all links of a given HTML page."""
    def __init__(self):
        super().__init__()
        self.links = []

    def error(self, arg1):
        print("Error while parsing HTML page",arg1)
        sys.exit(5)

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            attrs = dict(attrs)
            if 'href' not in attrs:
                return
            self.links.append(attrs['href'])

def print_percentage(percentage):
    """Print percentage of download to a terminal."""
    sys.stdout.write('\r' + ' ' * 40 + '\r') # user might have typed something
    sys.stdout.write(('%.1f %%' % percentage).rjust(7))
    sys.stdout.flush()



def get_links(url):
    with urllib.request.urlopen(url) as u:
        data = u.read().decode('utf-8')
    le = LinkExctractor()
    le.feed(data)
    return le.links

def download_to(url, target_file):
    """Download given url to given directory."""
    fetch_at_once = 150000
    with urllib.request.urlopen(url) as u:
        size = int(u.info().get('Content-Length'))
        with open(target_file, 'wb') as f:
            read = u.read(fetch_at_once)
            downloaded_so_far = 0
            while read:
                f.write(read)
                downloaded_so_far += len(read)
                print_percentage(downloaded_so_far / (size / 100))
                read = u.read(fetch_at_once)


def main(base_url, language, target):
    """Retrieve all zip files for the specified language and save them into
    `target`."""
    is_language = re.compile(r'^[A-Z]{3}/?$')
    # a language is identified by three letters in upper case.
    langs = [l.rstrip('/') for l in get_links(base_url) if is_language.search(l)]
    try:
        index = [l.lower() for l in langs].index(language.lower())
    except ValueError:
        print('Language', language, 'not found, the following languages exist:',
                ' '.join(langs))
        sys.exit(3)

    if not os.path.exists(target):
        os.makedirs(target)

    lang_url = '/'.join((base_url, langs[index]))
    for download in get_links(lang_url):
        if not download.lower().endswith('.zip'):
            continue # ignore non-zip files
        src_url = '/'.join((lang_url, download))
        target_zip = os.path.join(target, download)
        print("Downloading",target_zip)
        download_to(src_url, target_zip)
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

