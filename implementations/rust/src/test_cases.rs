use crate::marc_to_json;

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
    pretty_assertions::assert_eq!(marc_to_json(input).unwrap(), expected_json)
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
fn parse_error_1() {
    let input = r#"
.x.y 1
"#
    .trim();
    pretty_assertions::assert_eq!(marc_to_json(input).err().unwrap(), " --> 1:6
  |
1 | .x.y 1
  |      ^---
  |
  = expected COMMENT, array_access_new, array_access_last, tuple_access_new, tuple_access_last, object_access, or map_access")
}

#[test]
fn error_duplicate_assignment_1() {
    let input = r#"
.x = 2
.x = 3
"#
    .trim();
    pretty_assertions::assert_eq!(
        marc_to_json(input).err().unwrap(),
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
        marc_to_json(input).err().unwrap(),
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
        marc_to_json(input).err().unwrap(),
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
