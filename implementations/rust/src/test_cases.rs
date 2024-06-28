use crate::{format_merc, json_to_merc_string, merc_to_json, merc_to_json_string, parser::parse};

#[test]
fn merc_to_json_1() {
    let input = r#"
    
# Numbers (identical to JSON numbers)
.pi = 3.141592653
.sextillion = -6.02e+23

# Map
.materials{metal}.reflectivity = 1.0
.materials{metal}.metallic = true
.materials{plastic}.reflectivity = 0.5
.materials{"Infinity stones"}."soul affinity" = "fire"


# Array of objects (using explicit keys)
# These user-defined keys are solely to construct the array
# They are not consumable by application code
.excludes[+] = "node_modules/"
.excludes[+] = "dist/"
.excludes[+] = "target/" 

# Map
# Map and object are identical implementation-wise
# But map keys signify the reader that they are user-defined
# instead of schema-defined
.dependencies{"@types/react-markdown"} = "~0.2.3"
.dependencies{graphql} = "1.2.3"
.dependencies{react}.name = "^0.1.0"

# Singleline Escaped String
.poem = "Lorem\nIpsum"

# Multiline-able Escaped string
.escaped-one-line = """"Look at me" I can contain single quote!"""
.escaped-multiline = """
I must start and end with a newline.
Otherwise it would be an error.
The first and last newline will be omitted in the constructed string.
"""

# Singleline Raw String
.path = '\n is not escaped'

# Multiline raw string
.description = '''

'Hello there!'
These are common materials.
They are stored in C:\SolarSystem:\Earth

'''

"#
    .trim();
    let expected_json = serde_json::json!({
      "pi": 3.141592653,
      "sextillion": -6.02e23,
      "dependencies": {
        "@types/react-markdown": "~0.2.3",
        "graphql": "1.2.3",
        "react": {
          "name": "^0.1.0"
        }
      },
      "description": "\n'Hello there!'\nThese are common materials.\nThey are stored in C:\\SolarSystem:\\Earth\n",
      "escaped-multiline": "I must start and end with a newline.\nOtherwise it would be an error.\nThe first and last newline will be omitted in the constructed string.",
      "escaped-one-line": "\"Look at me\" I can contain single quote!",
      "excludes": [
        "node_modules/",
        "dist/",
        "target/"
      ],
      "materials": {
        "Infinity stones": {
          "soul affinity": "fire"
        },
        "metal": {
          "metallic": true,
          "reflectivity": 1.0
        },
        "plastic": {
          "reflectivity": 0.5
        }
      },
      "path": "\\n is not escaped",
      "poem": "Lorem\nIpsum"
    });
    let actual: serde_json::Value =
        serde_json::from_str(&merc_to_json_string(input).unwrap()).unwrap();
    pretty_assertions::assert_eq!(actual, expected_json)
}

#[test]
fn json_to_merc_1() {
    let expected_merc = r#"
.description = """
These are common materials.
They are found on Earth.
"""
.entities[0].material = "metal"
.entities[0].name = "hero"
.entities[1].material = "plastic"
.entities[1].name = "monster"
.materials.metal.metallic = true
.materials.metal.reflectivity = 1.0
.materials.plastic.conductivity = null
.materials.plastic.reflectivity = 0.5
.scalarArray[+] = 1
.scalarArray[+] = 2
.scalarArray[+] = 3
"#
    .trim();
    let input = r#"{
      "materials": {
        "metal": { "reflectivity": 1.0, "metallic": true },
        "plastic": { "reflectivity": 0.5, "conductivity": null }
      },
      "entities": [
        { "name": "hero", "material": "metal" },
        { "name": "monster", "material": "plastic" }
      ],
      "scalarArray": [1,2,3],
      "description": "These are common materials.\nThey are found on Earth."
    }"#;
    pretty_assertions::assert_eq!(json_to_merc_string(input).unwrap(), expected_merc)
}

#[test]
fn top_level_object_1() {
    let input = r#"
.a.b.c = 123 
"#
    .trim();
    let expected_json = serde_json::json!({"a":{"b":{"c":123}}});
    pretty_assertions::assert_eq!(merc_to_json(input).unwrap(), expected_json)
}

#[test]
fn top_level_map_1() {
    let input = r#"
{a}{b}{c} = 123 
"#
    .trim();
    let expected_json = serde_json::json!({"a":{"b":{"c":123}}});
    pretty_assertions::assert_eq!(merc_to_json(input).unwrap(), expected_json)
}

#[test]
fn top_level_array_1() {
    let input = r#"
[0][0][0] = 1
[0][0][1] = 2
[0][1][2] = 3
[0][1][3] = 4
[1][2][4] = 5
"#
    .trim();
    let expected_json = serde_json::json!([[[1, 2], [3, 4]], [[5]]]);
    pretty_assertions::assert_eq!(merc_to_json(input).unwrap(), expected_json)
}

#[test]
fn array_order_1() {
    let input = r#"
[b].name = 1
[a].name = 2

[a].age = 3
[b].age = 4
"#
    .trim();
    let expected_json = serde_json::json!([{"name":1,"age":4},{"name":2,"age":3}]);
    pretty_assertions::assert_eq!(merc_to_json(input).unwrap(), expected_json)
}

#[test]
fn escaped_string() {
    let input = r#"
.x = "\"hello\n\""
"#
    .trim();
    let expected_json: serde_json::Value = serde_json::from_str(r#"{"x": "\"hello\n\""}"#).unwrap();
    pretty_assertions::assert_eq!(merc_to_json(input).unwrap(), expected_json)
}

#[test]
fn parse_error_1() {
    let input = r#"
.x.y 1
"#
    .trim();
    pretty_assertions::assert_eq!(
        merc_to_json_string(input).err().unwrap(),
        " --> 1:6
  |
1 | .x.y 1
  |      ^---
  |
  = expected array_access_implicit, array_access_explicit, object_access, or map_access"
    )
}

#[test]
fn error_duplicate_assignment_1() {
    let input = r#"
.x = 2
.x = 3
"#
    .trim();
    pretty_assertions::assert_eq!(
        merc_to_json_string(input).err().unwrap(),
        "
error: Duplicate Assignment
  |
1 | .x = 2
  |      - info: A value was previously assigned at this path.
2 | .x = 3
  |      ^ Attempting to assign a new value at the same path is not allowed.
  |"
        .trim_start()
    )
}

#[test]
fn error_type_mismatch_1() {
    let input = r#"
.x.y = 2
.x{z} = 3
"#
    .trim();
    pretty_assertions::assert_eq!(
        merc_to_json_string(input).err().unwrap(),
        "
error: Type Mismatch
  |
1 | .x.y = 2
  |   -- info: The type of the parent value was first inferred as Object due to this access.
2 | .x{z} = 3
  |   ^^^ Error: this access treats the parent value as Map, but it was inferred as a different type.
  |"
        .trim_start()
    )
}

#[test]
fn error_multiline_string_not_starting_with_newline() {
    let input = r#"
[+] = '''hello
'''"#
        .trim();
    pretty_assertions::assert_eq!(
        merc_to_json_string(input).err().unwrap(),
        " 
error: Incorrect multi-line string format
  |
1 |   [+] = '''hello
  |  _______^
2 | | '''
  | |___^ The content of a multiline string should start with a newline
  |
"
        .trim()
    )
}

#[test]
fn error_multiline_string_not_ending_with_newline() {
    let input = r#"
[+] = '''
hello'''"#
        .trim();
    pretty_assertions::assert_eq!(
        merc_to_json_string(input).err().unwrap(),
        "
error: Incorrect multi-line string format
  |
1 |   [+] = '''
  |  _______^
2 | | hello'''
  | |________^ The content of a multiline string should end with a newline
  |
"
        .trim()
    )
}

#[test]
fn format_merc_1() {
    let input = r#"
# Map

# Yes
.materials{metal}.reflectivity = 1.0
.materials{"plastic"}.reflectivity = 0.5
.materials{metal}.metallic = true
.materials{plastic}.conductivity = null
.materials{"Infinity stones"}."soul affinity" = "fire"

# Array of objects
.entities[hero].name = "hero"
.entities[hero].material = "metal"

.entities[monster].material = "plastic"
.entities[monster].name = "monster"

# Multiline string
.description = """
These \u1234 are common materials.
They are found on Earth.
"""

"#
    .trim();
    let expected = r#"
# Multiline string
.description = """
These ሴ are common materials.
They are found on Earth.
"""
.entities[hero].material = "metal"

# Array of objects
.entities[hero].name = "hero"
.entities[monster].material = "plastic"
.entities[monster].name = "monster"
.materials{"Infinity stones"}."soul affinity" = "fire"
.materials{metal}.metallic = true

# Map
# Yes
.materials{metal}.reflectivity = 1.0
.materials{plastic}.conductivity = null
.materials{plastic}.reflectivity = 0.5
"#
    .trim();
    let actual = format_merc(input).unwrap();

    pretty_assertions::assert_eq!(actual, expected);

    // Test reciprocity:
    // format(parse(format(merc))) === format(merc)
    assert_eq!(
        format_merc(
            &parse(&format_merc(input).unwrap())
                .unwrap()
                .into_string()
                .unwrap()
        )
        .unwrap(),
        format_merc(input).unwrap()
    );

    // Test idempotency:
    // format(format(merc)) === format(merc)
    assert_eq!(
        format_merc(&format_merc(input).unwrap()).unwrap(),
        format_merc(input).unwrap()
    );
}
