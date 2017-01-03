import os
import re
import shutil
import string
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

def strip_punctuation(data):
    """Strip non-alphabetical characters from given data."""
    newstr = []
    cached_word = []
    # the last character is a space / newline, if newstr does not end on such a
    # charcter and cached_word is empty
    last_was_newline = lambda: (not newstr or newstr[-1] == '\n') and not cached_word
    last_was_space = lambda: (not newstr or newstr[-1].isspace()) and not cached_word
    has_next = lambda x: x < len(data)-1

    for index, c in enumerate(data):
        if c.isalpha():
            cached_word.append(c)
        elif c.isspace():
            # keep a \n, but not multiple ones
            if c == '\n' and not last_was_newline():
                # add cached word, if any
                if cached_word:
                    newstr.append(cached_word[0].lower())
                    newstr.extend(cached_word[1:])
                    cached_word.clear()
                newstr.append('\n')
            elif not last_was_space():
                newstr.extend(cached_word)
                cached_word.clear()
                newstr.append(' ')
        elif c == "'" or c == "’":
            if cached_word: # apostrophe in word is ok
                cached_word.append(c)
            else: # discard whole word
                continue
        elif c in string.punctuation:
            # strip punctuation at end of words
            if not has_next(index) or (has_next(index) and
                    data[index+1].isspace()):
                continue # ignore this punctuation; won't work for quotes and
                    # punctuation, but that doesn't  occur in the europarl data
        else:
            cached_word.clear() # words with non-alphabetical characters are completely ignored

    newstr.extend(cached_word)
    return ''.join(newstr)


def extract_data(language, source_tgz):
    """Extract source_tgz and move the text file with sentences from <language>
    to target."""
    with tarfile.open(source_tgz) as tar:
        for entry in tar:
            if entry.path.endswith('.%s' % language):
                f = tar.extractfile(entry)
                return f.read().decode('utf-8')


def main():
    if len(sys.argv) != 3:
        print("Usage: %s <language code> <target corpus file>" % sys.argv[0])
        print("\nThis script downloads the proceedings of the European Parliament in already preprocessed format.")
        print(("The language has to be  a two-digit code. The target corpus is the output file "
            "of CRAFT, to which it is appended or which is created.\n"
            "In traditional word2vec setups, this file is sometimes called 'text8'"))
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
    if os.path.exists(target) and not os.path.isfile(target):
        print("Error: Target must be a path to a file")
        sys.exit(3)
    tmpdir = tempfile.mkdtemp()
    try:
        tmpdest = os.path.join(tmpdir, 'tmp.tgz')
        print("Downloading %s to %s" % (link, tmpdest))
        common.download_to(link, tmpdest)
        data = extract_data(lang, tmpdest)
        with open(target, 'a') as f:
            f.write('\n')
            f.write(strip_punctuation(data))
        print("Extracted ile to", target)
    except:
        if tmpdir:
            shutil.rmtree(tmpdir)
        print()
        raise

if __name__ == '__main__':
    main()
