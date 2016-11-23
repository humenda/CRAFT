"""Download all ebooks from the Gutenberg project with a specified language.

It is not possible to download the books from Gutenberg directly, but they can
be retrieved from a mirror.
A list can be found at http://www.gutenberg.org/MIRRORS.ALL.

When the program is run, /GUTINDEX.ZIP and /ls-R are downloaded and parsed.

GUTINDEX.ALL:
A book is made up of a non-indented line containing the book title and book
number, followed by lines with attributes. Attributes may be indented or
unindented and even omitted. Therefore the only reliable book identification is
the title line, made up of the title and the book number.
Books without the `[Language: xyz]` attribute are in English.

To retrieve the path to the ebook, the file /ls-R is parsed. It is formatted, as
the name suggest, like the output of `ls -R`. This script only considers plain
text files.

After text files have been retrieved, they are encoded in UTF-8 and those who
are automatically detected to be non-free (free as in freedom) are removed."""


import datetime
import os
import re
import sys
import urllib.request
import zipfile

MIRROR = "http://www.mirrorservice.org/sites/ftp.ibiblio.org/pub/docs/books/gutenberg/"

def is_download_required(path):
    """Return True if file is older than two days or doesn't exist."""
    if not os.path.exists(path):
        return True
    two_days_ago = datetime.datetime.now() - datetime.timedelta(days=2)
    modification = datetime.datetime.fromtimestamp(os.path.getmtime(path))
    return modification < two_days_ago

def retrieve(url_path, outputfilename):
    """Fetch file from MIRROR/url_path and save it to outputfilename."""
    url = '/'.join((MIRROR, url_path))
    with open(outputfilename, 'wb') as f:
        try:
            with urllib.request.urlopen(url) as u:
                f.write(u.read())
        except urllib.error.HTTPError as e:
            # add URL to error
            if e.code == 404:
                e.msg = '%s; url: %s' % (e.msg, url)
            raise e


# these are local to the function below
copyright_regexes = [re.compile('COPYRIGHT PROTECTED'),
        re.compile('(?:t|T)his.*COPYRIGHTED (?:P|p)roject')]

def remove_copyrighted(file_name):
    """Try to detect copyrighted books and purge them. Return True, if the book
    was removed."""
    with open(file_name) as f:
        text = f.read()
    for expr in copyright_regexes:
        if expr.search(text):
            os.remove(file_name)
            return True
    return False

def parse_book_index(path):
    """Parse the GUTINDEX.ALL file and extract book number, language and title
    of each book. For a description of the file format, see the module doc
    string."""
    def content_iterator(lines):
        """Skip the preamble of the document."""
        in_preamble = True
        for lnum, ln in enumerate(lines):
            # strip \n and non-breaking spaces
            ln = ln.replace("\xA0", " ").rstrip()
            if in_preamble: # skip file format specification
                if ln.startswith("TITLE and AUTHOR") and ln.endswith("ETEXT NO."):
                    in_preamble = False
                else:
                    continue

            if ln.startswith("<==End of GUTINDEX.ALL"):
                break # end of document

            yield (lnum, ln) # yield each content line


    ebooks = {} # number -> (language, title)
    title = None
    ebook_no = None
    lang_pattern = re.compile(r"\[Language: (\w+?)\]")
    title_pattern = re.compile(r"""^
        (.*?) # match any title
        (?:\s+|\)|\]) # match spaces or closing parenthesis (before book number)
        (\d+[A-Z]?)$""", # book number, optionally ending on B or C
        re.VERBOSE)
    for lnum, line in content_iterator(open(path, encoding="utf8")):
        if not line.strip():
            title, ebook_no = None, None # drop old parsed book title/number
            continue # Ignore empty lines.


        # attribute line, try to extract language
        if line[0].isspace() or line.startswith('['):
            res = lang_pattern.search(line)
            if not res:
                continue #  no language attribute
            language = res.group(1)
            if not title: # title and ebook number are extracted at the same time
                print("Line {}: language pattern matched, but no title found!".\
                        format(lnum+1))
            else:
                ebooks[ebook_no] = (language, title)
                ebook_no, title = None, None
        else: # title line?
            is_title = title_pattern.search(line)
            if is_title:
                # if title / number found and there are already some, that means
                # no language information found and previous book was an English
                # book, flush old information first
                if title and ebook_no:
                    ebooks[ebook_no] = ('English', title)
                    title, ebook_no = None, None
                # extract current information
                title, ebook_no = is_title.groups()
                ebook_no = int(ebook_no.rstrip('B').rstrip('C'))
    return ebooks



def parse_file_index(path):
    """Parse the ls-R file to obtain a list of plain text books available."""
    ebook2path = {}
    fname_pattern = re.compile(r'^(\d+)(-[0-9]|-[a-z])?(\.[a-z]+)$')
    directory = None # directory name which is first line of a ls -R stanza
    ignore_until_next_paragraph = False
    for line in (l.rstrip() for l in open(path)):
        if not line:
            ignore_until_next_paragraph = False
            continue # skip empty lines
        elif ignore_until_next_paragraph:
            continue # ignore all lines in an "old" directory

        # is it a directory:
        if line.startswith("./") and line.endswith(':'):
            line = line[2:-1]
            if line.endswith('old'):
                ignore_until_next_paragraph = True
            else:
                directory = line
        else: # it's a file
            if not line.endswith('.txt'):
                continue
            ebook_no= fname_pattern.search(line)
            # ignore non-ebook files
            if not ebook_no:
                continue
            ebook_no, suff = ebook_no.groups()[:2]
            ebook_no = int(ebook_no)
                # we only want .txt files or files with no ending
            # add ebook if not present already and if a directory has been found before
            if ebook_no not in ebook2path and directory:
                ebook2path[ebook_no] = '/'.join((directory, line))
            # also add/overwrite if it ends on -8.txt, because these are UTF-8
            elif suff == '-8':
                ebook2path[ebook_no] = '/'.join((directory, line))
    return ebook2path

def recode_file(path):
    """Convert a file into UTF-8. Unfortunately, some books are not properly
    encoded, so try to catch these cases. I don't know of a better way of
    determining the encoding, so it's try-and-fail."""
    def read(fpath, src_enc):
        with open(fpath, 'r', encoding=src_enc) as f:
            return f.read()

    def write(fpath, data):
        if '\r' in data:
            data = data.replace('\r', '')
        with open(fpath, 'w', encoding="UTF-8") as f:
            f.write(data)

    def load_as_unicode(fn):
        try:
            data = read(fn, 'UTF-8')
            return data # UTF-8, fine
        except UnicodeDecodeError:
            pass # see below
        # if we're here, try latin1 next
        try:
            data = read(path, 'ISO-8859-1')
        except UnicodeDecodeError:
            pass
        return data
    data = load_as_unicode(path)
    # strip \r
    write(path, data.replace('\r', ''))


    # if we're here, we couldn't figure out the encoding
    print("Warning: couldn't determine encoding of", path)


def main(language, output_directory):
    # ensure directories exist.
    meta_dir = os.path.join(output_directory, 'meta')
    for directory in (output_directory, meta_dir):
        if not os.path.exists(directory):
            os.mkdir(directory)

    # download book index, unzip it.
    book_index_path = os.path.join(meta_dir, "GUTINDEX.zip")
    if is_download_required(book_index_path):
        retrieve("GUTINDEX.zip", book_index_path)
        zipfile.ZipFile(book_index_path).extractall(meta_dir)
        os.remove(book_index_path)
        # GUTINDEX.ZIP contains GUTINDEX.ALL, so rename the path
    book_index_path = book_index_path.rstrip('zip') + 'ALL'

    # download ebook file index
    file_index_path = os.path.join(meta_dir, "ls-R")
    if is_download_required(file_index_path):
        retrieve("ls-R", file_index_path)

    bookno2meta = parse_book_index(book_index_path)
    bookno2path = parse_file_index(file_index_path)

    wanted_books = sorted(no for no, (lang, title) in bookno2meta.items()
            if lang == language)

    print("%d books in %s found." % (len(wanted_books), language))

    books_not_available = 0
    for book_no in wanted_books:
        url_path = bookno2path.get(book_no)
        if not url_path: # no plain text version available
            books_not_available += 1
            continue

        name = url_path.rstrip('/').split('/')[-1]
        output_name = os.path.join(output_directory, name)
        if os.path.exists(output_name):
            continue # skip files which were downloaded
        else:
            print("Downloading %d: %s" % (book_no, bookno2meta[book_no][1]))
            retrieve(url_path, output_name)
            recode_file(output_name)
            if remove_copyrighted(output_name):
                print("%s is copyrighted, removed." % output_name)
    print(("%d books didn't have a download candidate (bug in the file index "
            "parser, so they were skipped.") % books_not_available)


if __name__ == '__main__':
    if not len(sys.argv) == 3:
        print(("Error: not enough parameters\nUsage: %s <language> <path>\n\n"
            "<language> can be any language from the Gutenberg project. It has "
            "to be in English. Examples: English, French, German\n"
            "<path> is the output directory with all text files."))
        sys.exit(1)
    else:
        main(sys.argv[1], sys.argv[2])

