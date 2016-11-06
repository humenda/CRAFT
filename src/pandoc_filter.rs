/// Filter the Pandoc AST for plain text
///
/// Pandoc parses the document into an abstract syntax tree (AST), which can represent the whole
/// document. Objects (JSON) always consist of `"t":"some_type"` and 
/// `"c":`content`. Content can either be another object or most likely, a JSON array. This module
/// parses the plain text bits out of this AST.
use json;
use json::*;

/// Handle all different kind of pandoc objects in a Pandoc AST (e.g. Header or Str)
fn handle_pandoc_entities(output: &mut String, entity: &mut object::Object) {
    // every pandoc object consists of a "t" (type) and a content "c"; match on the type:
    match entity.get("t").unwrap_or_else(|| panic!("broken json")).to_string().as_ref() {
        // add a space, if last character wasn't already a space
        "Space" | "SoftBreak" | "LineBreak" => match output.chars().rev().next() {
            Some(x) if !x.is_whitespace() => output.push(' '),
            _ => ()
        },

        // use take_string to extract the string of this element
        "Str" => if let Some(x) = entity.get_mut("c") {
            output.push_str(x.take_string().unwrap().clone().as_ref());
        },

        // handle heading; third element  contains content
        "Header" => if let Some(heading) = entity.get_mut("c") {
            recurse_json_tree(output, &mut heading[2]); // 2nd element of array contains content
        },

        // these all contain JsonValue::Array, so better process them with recursion
        "Para" | "Plain" | "BlockQuote" | "BulletList" | "DefinitionList" |
                "Emph" | "Strong" | "Strikeout" | "SmallCaps" | "Note" =>
            if let Some(thing) = entity.get_mut("c") {
                recurse_json_tree(output, thing);
        },

        // these are arrays with two items, where the first are attributes
        t @ "OrderedList" | t @ "Div" | t @ "Quoted" | t @ "Span" =>
            if let Some(array) = entity.get_mut("c") {
                match *array {
                    JsonValue::Array(ref mut x) if x.len() == 2 =>
                        recurse_json_tree(output, &mut x[1]),
                    _ => panic!("{}: expected a JSON array with length 2, got: {}",
                                t, array),
            }
        },

        // these have a JsonValue::Array with thee values, where the second is content
        y @ "Link" | y @ "Image" => if let Some(thing) = entity.get_mut("c") {
            match *thing {
                JsonValue::Array(ref mut x) if x.len() == 3 =>
                    recurse_json_tree(output, &mut x[1]),
                _ => panic!("{}: expected a JSON array, got: {}", y, thing),
            }
        },

        // types to ignore
        "CodeBlock" | "RawBlock" | "HorizontalRule" | "Table" | "Superscript" |
                "subscript" | "Cite" | "Code" | "Math" | "RawInline" => (),
        _ => panic!("Unknown type identifier found: {:?}", entity),
    }
}

/// recursively extract all string parts
fn recurse_json_tree(output: &mut String, jsval: &mut JsonValue) {
    match jsval {
        &mut JsonValue::Null => return,
        &mut JsonValue::Short(data) => output.push_str(data.as_str().into()),
        &mut JsonValue::String(ref mut data) => output.push_str(data.as_str().into()),
        &mut JsonValue::Number(data) => output.push_str(data.to_string().as_str()),
        &mut JsonValue::Boolean(data) => output.push_str(data.to_string().as_str()),
        &mut JsonValue::Object(ref mut entity) => handle_pandoc_entities(output, entity),
        &mut JsonValue::Array(ref mut values) => for mut val in values.iter_mut() {
            recurse_json_tree(output, &mut val);
        },
    }
}

pub fn stringify_text(pandoc_dump: String) -> String {
    let mut ast = json::parse(&pandoc_dump).unwrap();
    let mut output = String::new();
    recurse_json_tree(&mut output, &mut ast);
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

