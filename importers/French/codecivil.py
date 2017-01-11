"""This importer script downloads the code civil in markdown format and prepares
it for processing by CRAFT."""

import os
import shutil
import sys
import tempfile


GIT_URL = "https://github.com/steeve/france.code-civil"

def main(target):
    if os.path.exists(target):
        print("Error: target may not exist yet.")
        sys.exit(1)

    tmpdir = tempfile.mkdtemp()
    try:
        ret = os.system('git clone %s %s' % (GIT_URL, tmpdir))
        if ret:
            print("Error, git exited with exit status %d" % ret)
            sys.exit(4)
        shutil.rmtree(os.path.join(tmpdir, '.git'))
        gitignore = os.path.join(tmpdir, '.gitignore')
        if os.path.exists(gitignore):
            os.unlink(gitignore)
        shutil.move(tmpdir, target)
    except:
        shutil.rmtree(tmpdir)
        raise



if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: %s <targetdir>" % sys.argv[0])
        print(("\nThis script downloads the code civil to the specified target "
            "directory.\nThe target directory may not exist yet."))
        sys.exit(2)
    if not shutil.which('git'):
        print(("Error: git could not be found, please install it and make sure "
            " that it can be called from the command line."))
        sys.exit(3)
    main(sys.argv[1])

