CRAFT — CorpoRA-based Freedict Translations
=============================================

The goal of this program is to provide a corpus generator to feed lots of text
data into word2vec to train a natural language model. This model is then used to
enhance translation searches. For more information see the
[Wikipedia article](https://en.wikipedia.org/wiki/Word2vec).

This program is not finished yet. At the moment, it can import Wikipedia
articles, Gutenberg books and articles from Europeana. More corpora are planned. They have to be free, of
course.

A second part will take care of parsing the natural language model and do the
clever translation suggestions. The artificial language step in between is still
carried out by the original word2vec software.

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

The Europeana module is experimental. The texts from the project are in general
in quite a bad state, because they have been scanned using a OCR software and
contain quite a lot of errors.

