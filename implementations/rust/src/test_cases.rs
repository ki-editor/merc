use crate::{format_marc, json_to_marc_string, marc_to_json, marc_to_json_string, parser::parse};

#[test]
fn marc_to_json_1() {
    let input = r#"
# Map
.materials{metal}.reflectivity = 1.0
.materials{metal}.metallic = true
.materials{plastic}.reflectivity = 0.5
.materials{plastic}.conductivity = null
.materials{"Infinity stones"}."soul affinity" = "fire"

# Array of objects
.entities[i].name = "hero"
.entities[ ].material = "metal"

.entities[i].name = "monster"
.entities[ ].material = "plastic"

# Multiline string
.description = """
These are common materials.
They are found on Earth.
"""

"#
    .trim();
    let expected_json = serde_json::json!({
      "materials": {
        "metal": { "reflectivity": 1.0, "metallic": true },
        "plastic": { "reflectivity": 0.5, "conductivity": null },
        "Infinity stones": { "soul affinity": "fire" }
      },
      "entities": [
        { "name": "hero", "material": "metal" },
        { "name": "monster", "material": "plastic" }
      ],
      "description": "These are common materials.\nThey are found on Earth."
    });
    let actual: serde_json::Value =
        serde_json::from_str(&marc_to_json_string(input).unwrap()).unwrap();
    pretty_assertions::assert_eq!(actual, expected_json)
}

#[test]
fn json_to_marc_1() {
    let expected_marc = r#"
.description = """
These are common materials.
They are found on Earth.
"""
.entities[i].material = "metal"
.entities[ ].name = "hero"
.entities[i].material = "plastic"
.entities[ ].name = "monster"
.materials.metal.metallic = true
.materials.metal.reflectivity = 1.0
.materials.plastic.conductivity = null
.materials.plastic.reflectivity = 0.5
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
      "description": "These are common materials.\nThey are found on Earth."
    }"#;
    pretty_assertions::assert_eq!(json_to_marc_string(input).unwrap(), expected_marc)
}

#[test]
fn top_level_object_1() {
    let input = r#"
.a.b.c = 123 
"#
    .trim();
    let expected_json = serde_json::json!({"a":{"b":{"c":123}}});
    pretty_assertions::assert_eq!(marc_to_json(input).unwrap(), expected_json)
}

#[test]
fn top_level_map_1() {
    let input = r#"
{a}{b}{c} = 123 
"#
    .trim();
    let expected_json = serde_json::json!({"a":{"b":{"c":123}}});
    pretty_assertions::assert_eq!(marc_to_json(input).unwrap(), expected_json)
}

#[test]
fn top_level_array_1() {
    let input = r#"
[i][i][i] = 1
[ ][ ][i] = 2
[ ][i][i] = 3
[ ][ ][i] = 4
[i][i][i] = 5
"#
    .trim();
    let expected_json = serde_json::json!([[[1, 2], [3, 4]], [[5]]]);
    pretty_assertions::assert_eq!(marc_to_json(input).unwrap(), expected_json)
}

#[test]
fn top_level_tuple_1() {
    let input = r#"
(i)(i)(i) = 1
( )( )(i) = 2
( )(i)(i) = 3
( )( )(i) = 4
(i)(i)(i) = 5
"#
    .trim();
    let expected_json = serde_json::json!([[[1, 2], [3, 4]], [[5]]]);
    pretty_assertions::assert_eq!(marc_to_json(input).unwrap(), expected_json)
}

#[test]
fn escaped_string() {
    let input = r#"
.x = "\"hello\n\""
"#
    .trim();
    let expected_json: serde_json::Value = serde_json::from_str(r#"{"x": "\"hello\n\""}"#).unwrap();
    pretty_assertions::assert_eq!(marc_to_json(input).unwrap(), expected_json)
}

#[test]
fn parse_error_1() {
    let input = r#"
.x.y 1
"#
    .trim();
    pretty_assertions::assert_eq!(marc_to_json_string(input).err().unwrap(), " --> 1:6
  |
1 | .x.y 1
  |      ^---
  |
  = expected array_access_new, array_access_last, tuple_access_new, tuple_access_last, object_access, or map_access")
}

#[test]
fn error_duplicate_assignment_1() {
    let input = r#"
.x = 2
.x = 3
"#
    .trim();
    pretty_assertions::assert_eq!(
        marc_to_json_string(input).err().unwrap(),
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
        marc_to_json_string(input).err().unwrap(),
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
fn error_last_array_element_not_found_1() {
    let input = r#"
.x[ ] = 2
"#
    .trim();
    pretty_assertions::assert_eq!(
        marc_to_json_string(input).err().unwrap(),
        "
error: Last Array Element Not Found
  |
1 | .x[ ] = 2
  |   ^^^ Last array element not found.
  |   --- help: Change `[ ]` to `[i]`
  |
"
        .trim()
    )
}

#[test]
fn format_marc_1() {
    let input = r#"
# Map

# Yes
.materials{metal}.reflectivity = 1.0
.materials{"plastic"}.reflectivity = 0.5
.materials{metal}.metallic = true
.materials{plastic}.conductivity = null
.materials{"Infinity stones"}."soul affinity" = "fire"

# Array of objects
.entities[i].name = "hero"
.entities[ ].material = "metal"

.entities[i].material = "plastic"
.entities[ ].name = "monster"

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
.entities[i].material = "metal"

# Array of objects
.entities[ ].name = "hero"
.entities[i].material = "plastic"
.entities[ ].name = "monster"
.materials{"Infinity stones"}."soul affinity" = "fire"
.materials{metal}.metallic = true

# Map
# Yes
.materials{metal}.reflectivity = 1.0
.materials{plastic}.conductivity = null
.materials{plastic}.reflectivity = 0.5
"#
    .trim();
    let actual = format_marc(input).unwrap();

    pretty_assertions::assert_eq!(actual, expected);

    // Test reciprocity:
    // format(parse(format(marc))) === format(marc)
    assert_eq!(
        format_marc(
            &parse(&format_marc(input).unwrap())
                .unwrap()
                .to_string()
                .unwrap()
        )
        .unwrap(),
        format_marc(input).unwrap()
    )
}
