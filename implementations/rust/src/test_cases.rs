use crate::{data::evaluate, marc_to_json, parser::parse};

#[test]
fn generic_example_1() {
    let input = r#"
# Map
.materials{metal}.reflectivity = 1.0
.materials{metal}.metallic = true
.materials{plastic}.reflectivity = 0.5
.materials{plastic}.conductivity = null

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
      },
      "entities": [
        { "name": "hero", "material": "metal" },
        { "name": "monster", "material": "plastic" }
      ],
      "description": "These are common materials.\nThey are found on Earth."
    });
    pretty_assertions::assert_eq!(marc_to_json(&input).unwrap(), expected_json)
}

#[test]
fn top_level_object_1() {
    let input = r#"
.a.b.c = 123 
"#
    .trim();
    let expected_json = serde_json::json!({"a":{"b":{"c":123}}});
    pretty_assertions::assert_eq!(marc_to_json(&input).unwrap(), expected_json)
}

#[test]
fn top_level_map_1() {
    let input = r#"
{a}{b}{c} = 123 
"#
    .trim();
    let expected_json = serde_json::json!({"a":{"b":{"c":123}}});
    pretty_assertions::assert_eq!(marc_to_json(&input).unwrap(), expected_json)
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
    pretty_assertions::assert_eq!(marc_to_json(&input).unwrap(), expected_json)
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
    pretty_assertions::assert_eq!(marc_to_json(&input).unwrap(), expected_json)
}
