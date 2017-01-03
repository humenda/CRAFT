"""This module groups functions to aid the download process of data files."""

from html.parser import HTMLParser
import sys
import urllib.request

def print_percentage(percentage):
    """Print percentage of download to a terminal. Expected is a is a float
    percentage >= 0 and <= 100."""
    sys.stdout.write('\r' + ' ' * 40 + '\r') # user might have typed something
    sys.stdout.write(('%.1f %%' % percentage).rjust(7))
    sys.stdout.flush()



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

def get_links(data, is_web_page=False):
    """Extract all links from the given first parameter. If `is_web_page=True`,
    the first parameter is interpreted as a HTML string."""
    if not is_web_page:
        with urllib.request.urlopen(data) as u:
            data = u.read().decode('utf-8')
    le = LinkExctractor()
    le.feed(data)
    return le.links

def download_to(url, target_file):
    """Download given url to given file."""
    fetch_at_once = 150000
    try:
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
        print()
    except urllib.error.HTTPError as u:
        if u.code == 404:
            u.msg += ', link: ' + url
            raise u



