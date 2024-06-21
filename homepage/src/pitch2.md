# Comparison

| Feature/Format           | MERC | JSON |             YAML              |           TOML            |
| ------------------------ | :--: | :--: | :---------------------------: | :-----------------------: |
| Supports Comments        |  ✅  |  ❌  |              ✅               |            ✅             |
| Single Representation    |  ✅  |  ✅  | ❌ (Due to Anchors & Aliases) | ❌ (Due to Inline tables) |
| Whitespace Insensitivity |  ✅  |  ✅  |              ❌               |            ✅             |
| Formatter Specification  |  ✅  |  ❌  |              ❌               |            ❌             |
| Map-Object Distinction   |  ✅  |  ❌  |              ❌               |            ❌             |
| Top-level Array          |  ✅  |  ✅  |              ✅               |            ❌             |

---

# Specification

For any aspects not explicitly defined within this MERC specification, refer to the [JSON Specification (ECMA-404)](https://ecma-international.org/publications-and-standards/standards/ecma-404/) as the default guideline.

# Specification (Syntax)

## Overview

A MERC configuration file is structured as a list of entries. Each entry represents a configuration directive and is composed of a path and a value.

## Entries

Each entry in a MERC file specifies a particular configuration setting. An entry has two components:

- **Path**: A sequence of accessors that defines where in the configuration hierarchy the value should be placed.
- **Value**: The actual data or setting that will be applied at the specified path.

Entries are written in the format `path = value`. The file can contain any number of entries, ordering is not important except for array elements.

## Paths

Paths can include any combination of object access, map access, array access, and tuple access. The type of the first access determines the root value type.

- **Object Access**: `.objectKey`
- **Map Access**: `{mapKey}`
- **Array Access**:
  - `[i]` specifically denotes a new element in an array and must use the key `i`.
  - `[ ]` denotes the last element in the array.
- **Tuple Access**:
  - `(i)` specifically denotes a new element in a tuple and must use the key `i`.
  - `( )` denotes the last element in the tuple.

## Keys

Keys are used for both the object accessor and the map accessor.
Keys must be alphanumeric and can include underscores (`_`) and dashes (`-`).
Quoted keys are also supported to allow for special characters such as whitespaces.
Note that quoted characters must be ASCII, non-ASCII character should be escaped using the `\u0000` format.
This is because there is no single acceptable lexicographic ordering exists for Unicode characters.
Newline should be escaped with `\n`.

Example of valid keys:

```bash
"hello world"
kebab-is-madeOfCamel_and_snake
0_me_start_with_digit
1238
```

## Values

Values must be scalar, identical to JSON scalar values, and include strings, booleans, numbers or null:

- **Strings**: Enclosed in double quotes. Multiline strings use triple double quotes, and surrounding whitespace is trimmed.
  - Example: `"Hello, World!"`
  - Multiline Example:
    ```python
    """
    This is a multiline string
    that spans multiple lines.
    """
    ```
- **Booleans**: `true` or `false`.
  - Example: `true`
- **Numbers**: Integers or decimals without quotes.
  - Integer Example: `42`
  - Decimal Example: `3.14`
- **Null** : `null`

## Whitespace Insensitivity

Whitespace is not sensitive in MERC; entries can be separated by any amount of whitespace. For example:

```bash
.x . y {   z } =    123
```

This entry is valid despite the irregular spacing.

## Comments

Comments provide context or explanations for an entry and must be placed on their own line directly above the entry they describe. Comments start with a `#` symbol.

- Example:
  ```bash
  # This comment describes the following setting for background color
  .settings.background = "blue"
  ```

# Specification (Semantics)

## 1. Root value type is inferred by the first entry

The root value type of a MERC configuration is determined by the first accessor in the first entry. This initial accessor sets the expected type for the root value.

In the following example, the root value type is inferred as Object because the first entry is `.x`, and `.x. implies the parent value is Object.

```python
.x{y} = 2
```

MERC allows any compound type to serve as the root value. This means that configurations can have top-level arrays, maps, and tuples.

## 2. Duplicated assignment is not allowed

Once a path is assigned a scalar value, it cannot be assigned another value.

```bash
.x.y = 1
.x.y = 2 # Error: Duplicated assignment
```

## 3. Type of value cannot be changed once defined

In the following example, the second line is erroneous because `.x` was inferred as `Object` due to `.y`, but the second line treats `.x` as `Map` via `{z}`.

```bash
.x.y = 1
.x{z} = 2 # Error
```

## 4. Array element must be initialized before being modified

The following example is erroneous because `.x` is not initialized with an element.

```bash
.x[ ].y = 123 # Error
```

To fix this, replace `[ ]` with `[i]`.

## 5. Entry order

Entry order is only important for array elements, any other entries can be freely ordered.
To demonstrate, both of the examples below are semantically equivalent:

```python
.foo[i].x = 1
.comment = "Hello"

.foo[ ].y = 2


.foo[i].x = 3
.foo[ ].y = 4
```

```python
.comment = "Hello"
.foo[i].y = 2
.foo[ ].x = 1

.foo[i].y = 4
.foo[ ].x = 3
```

# Specification (Formatter)

## Formatting Rules

A compliant MERC implementation must incorporate a formatter that adheres to the following rules to ensure consistency and readability:

## 1. **Equal Sign Spacing**

Surround the equal sign with exactly one space on both sides when separating a path from its value.

- Correct:

```python
 .setting = "value"
```

- Incorrect:

```python
.foo= "bar"
.baz  =  "spam"

```

## 2. **Whitespace Management**

The formatted output must be free of any leading or trailing whitespaces.

- Correct:

```python
.setting = "value"
```

- Incorrect:

```python

.setting = "value"

```

## 3. **Sorting Keys**

Within maps and objects, keys must be arranged in ascending lexicographical order, ignoring quotes for quoted keys.

- Correct:

```python
.apple = "fruit"
."cherry juice" = "fruit"
```

- Incorrect:

```python
."cherry juice" = "fruit"
.apple = "fruit"
```

## 4. **Array Integrity**

The original sequence of array elements must remain unchanged.
For example, the original config looks like this:

```python
.fruits[i] = "apple"

.fruits[i] = "banana"

```

- Correct formatting:

```python
.fruits[i] = "apple"
.fruits[i] = "banana"
```

- Incorrect formatting:

```python
.fruits[i] = "banana"
.fruits[i] = "apple"
```

## 5. **Comment Spacing**

An extra newline character should precede each comment group, if the comment group is preceded by an entry.

Group of comments should only be separated from each other by exactly one newline character.

- Correct:

  ```bash
  # Comment not preceded by entry should not have newlines above
  .foo = "bar"

  # This is a comment for the setting below
  # Another line of comment
  .setting = "value"
  ```

- Incorrect:

  ```bash
  .foo = "bar"
  # This is a comment for the setting below

  # Another line of comment
  .setting = "value"
  ```

  or

  ```bash
  .foo = "bar" # This is a comment for the setting below
  # Another line of comment
  .setting = "value"
  ```

  or

  ```bash

  # Comment not preceded by entry should not have newlines above
  .foo = "bar"
  ```

## 6. **Key Formatting**

Remove double quotes from keys when they are not necessary (i.e. the inner part of the key forms a valid identifier).

- Correct:
  ```python
  .setting = "value"
  ```
- Incorrect:
  ```python
  ."setting" = "value"
  ```

## 7. **Entry Separation**

Separate each entry by exactly one newline character, except for commented entries.

- Correct:

  ```bash
  .setting = "value"
  .anotherSetting = "anotherValue"
  ```

- Incorrect:

  ```bash
  .setting = "value"


  .anotherSetting = "anotherValue"
  ```

## 8. **Multiline String Formatting**

Format strings as multiline string literals if they contain any newline characters.

- Correct:
  ```python
  .greeting = """
  Hello,
  World!
  """
  ```
- Incorrect:
  ```python
  .greeting = "Hello,\nWorld!"
  ```

## 9. **Entry formatting**

Each entry should not contain any leading or trailing whitespaces.

- Correct:

  ```python
  .baz = "spam"
  ```

- Incorrect:
  ```python
     .baz = "spam"
  ```

# Specification (Metadata)

| Aspect         | Value              |
| -------------- | ------------------ |
| File Extension | `.marc`            |
| MIME type      | `application/marc` |
