# MaRC

MarC, the Maximumally Redundant Config Language.

## Objectives

1. Maximally redundant, no more getting lost in the long and deeply nested config file (like YAML and JSON)
2. No ambiguity, there exist only one kind of representation for each config disregarding comments (unlike TOML)
3. Easily formattable
4. Easy to modify (add/delete/update)
5. Easy to debug

## Example in JSON

```json
{
  "materials": {
    "metal": {
      "reflectivity": 1.0
    },
    "plastic": {
      "reflectivity": 0.5
    }
  },
  "entities": [
    {
      "name": "hero",
      "material": "metal"
    },
    {
      "name": "monster",
      "material": "plastic"
    }
  ],
  "description": "These are common materials.\nThey are found on Earth."
}
```

## Same example in MaRC

```
# Map
.materials{metal}.reflectivity = 1.0
.materials{plastic}.reflectivity = 0.5

# Array of objects
# Use [i] to push new element
.entities[i].name = "hero"
# Use [] to continue working on the same element
.entities[].material = "metal"

.entities[i].name = "monster"
.entities[].material = "plastic"

# Multiline string
.description = """
These are common materials.
They are found on Earth.
"""
```

## Grammar

```abnf
entry = entry-directive / entry-value / comment
entry-value = 1*access "=" value
access = access-map / access-array / access-object / access-tuple
access-map = "{" map-key "}"
map-key = identifier / string / integer
access-array = "[" ("i" / "j") "]"
access-object = "." identifier
access-tuple = "(" integer ")"

identifier = alphanumeric / quoted

value = string-singleline / string-multiline / integer / float / boolean / array-empty / map-empty / tuple-empty

entry-directive = "@" entry-value
comment = comment-singleline / comment-multiline
comment-singleline = "#" (?)
comment-multiline = "###" (?) "###"
```

# MARC

MARC, the MAximally Redundant Config language.

## Why MARC?

### 1. Easy to copy and paste

To demonstrate, we'll use the following example found from Github Actions documentation.

```yaml
on:
  push:
    paths:
      - "**.js"
```

You cannot just copy and paste this snippet into the following Github Actions config file and expect it to work.

You have to perform a **merge**, which is tedious and error-prone, and this becomes exponentially harder with the size of the existing config.

In this case, you should only copy `paths:\n      - '**.js'`, and paste it as the child of the `push` field.

```yaml
on:
  push:
    # Sequence of patterns matched against refs/heads
    branches:
      - main
      - "mona/octocat"
      - "releases/**"
    # Sequence of patterns matched against refs/tags
    tags:
      - v2
      - v1.*
```

Suppose Github Actions config uses MARC, the documentation example above would look as follows, and copy and pasting it becomes a no-brainer, because no **merging** is required.

```bash
on.push.paths[i] = "**.js"
```

###
