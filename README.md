CRAFT — CorpoRA generator for FreedicT
=============================================

Many artificial intelligence and data mining techniques require a lot of input
data to learn semantic relations between words. Word2vec is one such technique,
which is able to learn relations with unannotated input data. These trained word
relations can be then linked with FreeDict's dictionaries for different
applications.

This program is able to extract plain text for AI training from various sources.
It is work-in-progress and can import Wikipedia articles, Gutenberg books,
EU-DGT translation memories and articles from Europeana. More corpora are
planned. They have to be free, of course.

CRAFT is a library, which means you can easily re-use the functionality, i.e. to
post-process the data differently. For instance, this program assumes paragraphs
as a context by default, but other applications might require
sentence-granularity.


Requirements
------------

This program is written in Rust. You need Rust >= 1.13 to compile it. The
importers scripts require Python >= 3.3.

How It Works
------------

-   Input text is scraped by an importer script and serves as input to this
    program.

    Example:

    ```
    mkdir workspace
    python3 importers/wikipedia.py da workspace
    ```
-   The transformation has three steps:
    1.  preprocessing
        -   remove all formatting which the actual transformation cannot cope
            with
        -   remove all parts which would destroy text continuity, e.g. foot
            notes
    2.  call pandoc to transform the document into an abstract tree
        representation
    3.  extract text from abstract document tree, thereby stripping all
        formatting
    4.  remove all non-letter character from words (punctuation, quotes, ...)
        and remove the rest

    **Example:**

    ```
    target/release/crafted -w workspace/*.bz2
    ```


**Remarks:**

-   all input texts have to have \n line separators
    -   importer scripts take care of that
-   all input files must be encoded using UTF-8

A Word About The Importers
--------------------------

Wikipedia has the best quality of texts by far. Gutenberg books are quite good
in general, too, but can contain English text, although care has been taken to
prevent this.

The DGT translation memories are of high quality and offer an extensive range of European languages.

The Europeana module is experimental. The texts from the project are in general
in quite a bad state, because they have been scanned using a OCR software and
contain quite a lot of errors.

