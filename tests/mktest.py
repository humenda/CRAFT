"""Generate json serialized as rust string, together with some code to test it.
See README.test_pandoc_filter."""
import subprocess, sys

def call_pandoc(input, format):
    """Call pandoc with `input` as argument and return output in format `format`
    as string."""
    proc = subprocess.Popen(['pandoc', '-f', 'markdown', '-t', format],
            stdout=subprocess.PIPE, stdin=subprocess.PIPE)
    output = proc.communicate(input.encode(sys.getdefaultencoding()))[0]. \
            decode(sys.getdefaultencoding())
    return output

# name to insert into comment before - more of a laziness thing
name = input('Entity name: ')
# Markdown text to be transformed into AST
text = input('Markdown to serialize (literal \\n is replaced through proper \\n: ').replace('\\n', '\n')

# apply _some_ formatting to json output (not too much, so that it doesn't have
# 100 lines, but a bit, so that lines don't have > 800 characters
js_str = call_pandoc(text, 'json').replace('},', '},\n').replace('}],', '}],\n')
# reformat the stuff for Rust
js_str = '"%s".into();' % js_str.replace('"', '\\"').replace('\n', '\\\n      ')

plain = call_pandoc(text, 'plain').replace('\n', ' ').replace('  ', ' ').rstrip()

# print code
rust = ('    // this document contains a {} element\n'
        '    let json_str: String = {}\n    assert_eq!(call_filter(json_str), "{}");\n').\
            format(name, js_str, plain)

# I used it from a simplistic macro in vim, you might want to adjust this
with open('/tmp/u', 'w') as f:
    f.write(rust)

