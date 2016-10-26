//use pandoc::Pandoc;
//use pandoc_ast;
//use pandoc_ast::Block::*;
//use pandoc_ast::Inline;
//
//pub fn stringify_inline(text: Inline) -> Inline {
//}
//
//
//pub fn stringify_text(document_ast: &pandoc_ast::Pandoc) -> pandoc_ast::Pandoc {
//    for block in &mut document_ast.1 {
//        *block = match *block {
//            Plain(chunks) => Null,
//            Para(chunks) => Null,
//            CodeBlock(Attr, String) => Null,
//            RawBlock(Format, String) => Null,
//            // Vec<Block>:
//            BlockQuote(blocks) => Null,
//            // Ordered list (attributes and a list of items, each a list of blocks)
//            //OrderedList(ListAttributes, Vec<Vec<Block>>) => Null,
//            // Bullet list (list of items, each a list of blocks)
//            //BulletList(Vec<Vec<Block>>) => Null,
//            // Definition list Each list item is a pair consisting of a term (a list of inlines)
//            // and one or more definitions (each a list of blocks)
//            //DefinitionList(Vec<(Vec<Inline>, Vec<Vec<Block>>)>) => Null,
//            // Header - level (integer) and text (inlines)
//            Header(_, _, chunks) => Null,
//            HorizontalRule => Null,
//            /// Table, with caption, column alignments (required), relative column widths (0 = default),
//            // column headers (each a list of blocks), and rows (each a list of lists of blocks)
//            Table(_, _, _, _, _) => Null,
//            // Generic block container with attributes
//            Div(_, blocks) => Null,
//            // Nothing
//            Null => Null,
//        }
//    }
//    document_ast
//}
//
