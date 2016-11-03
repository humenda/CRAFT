use json;
use json::*;

fn handle_pandoc_entities(output: &mut String, entity: object::Object) {
    match entity {
        _ => (),
    }
}

/// recursively extract all string parts
fn recurse_json_tree(output: &mut String, jsval: JsonValue) {
    match jsval {
        JsonValue::Null => return,
        JsonValue::Short(data) => output.push_str(data.as_str().into()),
        JsonValue::String(data) => output.push_str(data.as_str().into()),
        JsonValue::Number(data) => output.push_str(data.to_string().as_str()),
        JsonValue::Boolean(data) => output.push_str(data.to_string().as_str()),
        JsonValue::Object(entity) => handle_pandoc_entities(output, entity),
        JsonValue::Array(values) => for val in values {
            recurse_json_tree(output, val);
        },
    }
}

pub fn stringify_text(pandoc_dump: String) -> String {
    let ast = json::parse(&pandoc_dump).unwrap();
    let mut output = String::new();
    recurse_json_tree(&mut output, ast);
    output
}
//    for block in &mut document_ast.1 {
//        *block = match *block {
//            Plain(chunks) => Null,
////            Para(chunks) => Null,
////            CodeBlock(Attr, String) => Null,
////            RawBlock(Format, String) => Null,
////            // Vec<Block>:
////            BlockQuote(blocks) => Null,
////            // Ordered list (attributes and a list of items, each a list of blocks)
////            //OrderedList(ListAttributes, Vec<Vec<Block>>) => Null,
////            // Bullet list (list of items, each a list of blocks)
////            //BulletList(Vec<Vec<Block>>) => Null,
////            // Definition list Each list item is a pair consisting of a term (a list of inlines)
////            // and one or more definitions (each a list of blocks)
////            //DefinitionList(Vec<(Vec<Inline>, Vec<Vec<Block>>)>) => Null,
////            // Header - level (integer) and text (inlines)
////            Header(_, _, chunks) => Null,
////            HorizontalRule => Null,
////            /// Table, with caption, column alignments (required), relative column widths (0 = default),
////            // column headers (each a list of blocks), and rows (each a list of lists of blocks)
////            Table(_, _, _, _, _) => Null,
////            // Generic block container with attributes
////            Div(_, blocks) => Null,
////            // Nothing
////            Null => Null,
//            _ => Null
//        }
//    }

