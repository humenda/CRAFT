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
        "Space" | "LineBreak" | "SoftBreak" => match output.chars().rev().next() {
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
        t @ "OrderedList" | t @ "Div" | t @ "Span" =>
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
                "Subscript" | "Cite" | "Code" | "Math" | "RawInline" | "Null" => (),
        _ => panic!("Unknown type pandoc AST identifier found: {:?}", entity),
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
        &mut JsonValue::Array(ref mut values) => {
            let lastindex = match values.len() {
                x if x > 0 => x - 1,
                _ => 0
            };
            for (i, mut val) in values.iter_mut().enumerate() {
                recurse_json_tree(output, &mut val);
                // between the items of an array are sometimes no spaces (e.g. in lists), so check and
                // insert a space
                match output.chars().rev().next() {
                    Some(x) if !x.is_whitespace() && i < lastindex =>
                        output.push(' '),
                    _ => ()
                };
            };
        }
    }
}

pub fn stringify_text(pandoc_dump: String) -> String {
    let ast = json::parse(&pandoc_dump).unwrap();
    let mut output = String::new();
    match ast {
        JsonValue::Array(mut values) => if values.len() == 2 {
            recurse_json_tree(&mut output, &mut values[1]);
        },
        _ => panic!("expected JSON document with an Array at top level object \
                    and two entries: unmeta and the contents of the parsed document.")
    };
    output
}

