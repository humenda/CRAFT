import os
import re
import shutil
import sys
import tarfile
import tempfile
import urllib.request

# import common module from same directory as this script is run from
sys.path.append(os.path.abspath(sys.argv[0]))
import common

CORPUS_PATTERN = re.compile(r'.*\/([a-z]{2})-[a-z]{2}.tgz$')
BASE_URL = 'http://www.statmt.org/europarl/'


def get_languages(page):
    languages = []
    for link in common.get_links(page, is_web_page=True):
        match = CORPUS_PATTERN.search(link)
        if match:
            languages.append(match.groups()[0])
    languages.append('en') # English is pivot language and hence not automatically added
    return languages

def get_link_for(lang, page):
    lastlink = None
    for link in common.get_links(page, is_web_page=True):
        match = CORPUS_PATTERN.search(link)
        if match:
            full_uri = '/'.join((BASE_URL, link))
            if match.groups()[0] == lang:
                return full_uri
            lastlink = full_uri
    if lang == 'en' and lastlink:
        return lastlink
    else:
        raise ValueError("Couldn't find download link for language " + str(lang))


def extract_and_move(language, source_tgz, dest_dir):
    """Extract source_tgz and move the text file with sentences from <language>
    to target."""
    with tarfile.open(source_tgz) as tar:
        for entry in tar:
            if entry.path.endswith('.%s' % language):
                tar.extract(entry, dest_dir)
                return os.path.join(dest_dir, entry.path.split('/')[-1])


def main():
    if len(sys.argv) != 3:
        print("Usage: %s <language code> <target directory>" % sys.argv[0])
        print("\nThis script downloads the proceedings of the European Parliament in already preprocessed format.")
        print("The language has to be  a two-digit code and the target is a directory.")
        sys.exit(1)
    lang = sys.argv[1]
    target = sys.argv[2]
    with urllib.request.urlopen(BASE_URL) as u:
        page = u.read().decode('utf-8')
    langs = get_languages(page)
    if lang not in langs:
        print("Unknown language, only the following are known: %s" % \
                ', '.join(langs))
        sys.exit(2)

    link = get_link_for(lang, page)
    if not os.path.exists(target) or not os.path.isdir(target):
        print("Error: Target must exist and must be a directory")
        sys.exit(3)
    tmpdir = tempfile.mkdtemp()
    try:
        tmpdest = os.path.join(tmpdir, 'tmp.tgz')
        print("Downloading %s to %s" % (link, tmpdest))
        common.download_to(link, tmpdest)
        target = extract_and_move(lang, tmpdest, target)
        print("Extracted ile to", target)
    except:
        if tmpdir:
            shutil.rmtree(tmpdir)
        print()
        raise

main()
