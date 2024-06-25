# Why MERC?

## 1. Fearless Copy-Paste

Integrating new snippets into configuration files is a common task that can be surprisingly complex. This complexity arises not from the format itself, whether YAML, JSON, or another, but from the need to maintain the logical structure and relationships within the configuration.

To correctly incorporate new content, you must perform a **merge**. Merging is the process of combining two sets of data—your existing configuration and the new snippet—into a cohesive whole. This often involves manually aligning indentation levels, resolving conflicts between keys, and ensuring that lists are properly extended rather than overwritten.

Consider this snippet intended for addition to an existing configuration:

```yaml
on:
  push:
    paths:
      - "**.js"
```

You might think you can copy and paste the entire snippet as is. However, if your configuration already contains related properties or nested elements, you must integrate only the relevant parts to maintain the integrity of your file.

Here's an example of an existing configuration where you would need to carefully insert the new `paths` property:

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

In this case, you should only copy `paths:\n      - '**.js'` and place it under the `push` field.

MERC eliminates this complexity by using a path-value syntax that allows for direct insertion without concern for structural conflicts:

```bash
on.push.paths[+] = "**.js"
```

With MERC, you can effortlessly copy and paste without needing to merge—**no complex integration required**. This simplicity minimizes errors and streamlines configuration updates.

## 2. Streamlined Change Review

The challenge of reviewing changes is amplified with large configuration files. Consider the JSON diff below; the necessity to scroll up and down makes it difficult to confidently identify which path in the config has been altered:

```diff
    "type": "spot",
-   "size": "t3-medium",
+   "size": "t3-large",
    "securityGroup": "internal"
```

MERC simplifies this process. Changes are immediately apparent, eliminating any guesswork:

```diff
    machines{api}.type = "spot"
-   machines{api}.size = "t3-medium"
+   machines{api}.size = "t3-large"
    machines{api}.securityGroup" = "internal"
```

## 3. Easily Decipherable

When attempting to comprehend a configuration file, it’s often necessary to mentally decode it into a format akin to MERC to fully understand its structure.

Creating configurations in MERC conserves cognitive energy for troubleshooting real issues, eliminating any uncertainty about the accuracy of your interpretation.

## 4. Natural Documentation

Documenting configuration schemas in hierarchical formats like JSON or YAML often lacks clarity and can be unintuitive. Ironically, most existing configuration documentations gravitate towards a MERC-like syntax for maximum clarity, as seen in these examples:

| Documentation                                                                                                           | MERC-like Syntax Example           |
| ----------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| [Gitlab CI/CD Yaml](https://docs.gitlab.com/ee/ci/yaml)                                                                 | `cache:key:files`                  |
| [Github Actions Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions) | `jobs.<job_id>.defaults.run.shell` |

This MERC-like syntax, while clear, often differs significantly from the actual configuration format, creating a disconnect that readers must bridge to fully grasp the function of each property. Additionally, documentation writers must invest considerable time and effort to create and maintain this pseudo MERC-like syntax, which is not required in the actual configuration files.

MERC, on the other hand, eliminates this disparity and extra effort. The syntax used in documentation is identical to that used in actual configurations—what you see is what you get. This one-to-one correspondence between documentation and implementation simplifies understanding and reduces the cognitive load.

## 5. Straightforward Communication

Navigating configuration errors in hierarchical formats can be as perplexing as giving directions in a labyrinth. It often involves convoluted explanations akin to guiding a tourist through a maze of streets: _"Proceed straight, then take a left at the souvenir shop, followed by a right turn…"_

In contrast, MERC’s simplicity turns these complex instructions into straightforward directions. It’s like pointing directly to the destination and saying, _"The restroom is right there."_

For example, [this StackOverflow answer](https://stackoverflow.com/a/7736755/6587634) (slightly modified) looks like this:

> I solved my problem with this.
>
> ```xml
> <httpErrors errorMode="Custom">
>   <remove statusCode="404" subStatusCode='-1' />
> </httpErrors>
> ```
>
> This needs to go in Web.config, under `<configuration>`, then `<system.webServer>`:
>
> e.g.
>
> ```xml
> <configuration>
>     <system.webServer>
>         <httpErrors ...>
>             // define errors in here ...
>         </httpErrors>
>     </system.webServer>
> </configuration>
> ```

If the configuration was in MERC instead of XML, the answer would look like this, without the navigational instructions:

> I solved my problem by adding the following lines:
>
> ```python
> configuration.system.webServer.httpErrors.remove[a].statusCode = 404
> configuration.system.webServer.httpErrors.remove[a].subStatusCode = -1
> ```

The same goes for communicating modifications or deletions of existing values in the configuration file.

## 6. Semantic Clarity

In contrast to many common configuration formats, MERC clearly differentiates between Objects and Maps in its syntax.

This distinction may seem subtle, but it becomes incredibly valuable in large configuration files. It provides clarity and assurance, allowing you to immediately recognize which properties are defined by the user and which are dictated by the schema.

For example, consider the following Nx.json config:

```json
{
  "targetDefaults": {
    "build": {
      "inputs": ["production", "^production"],
      "dependsOn": ["^build"],
      "options": {
        "main": "{projectRoot}/src/index.ts"
      },
      "cache": true
    },
    "test": {
      "cache": true,
      "inputs": ["default", "^production", "{workspaceRoot}/jest.preset.js"],
      "outputs": ["{workspaceRoot}/coverage/{projectRoot}"],
      "executor": "@nx/jest:jest"
    }
  }
}
```

In this snippet, it’s ambiguous whether `build` or `test` are defined by the user or are part of the predefined schema.
Similarly, `options.main` could raise questions about its origin.

MERC eliminates such ambiguities:

```python
.targetDefaults.{build}.cache = true
.targetDefaults.{build}.dependsOn[+] = "^build"
.targetDefaults.{build}.inputs[+] = "production"
.targetDefaults.{build}.inputs[+] = "^production"
.targetDefaults.{build}.options.main = "{projectRoot}/src/index.ts"
.targetDefaults.{test}.cache = true
.targetDefaults.{test}.executor = "@nx/jest:jest"
.targetDefaults.{test}.inputs[+] = "default"
.targetDefaults.{test}.inputs[+] = "^production"
.targetDefaults.{test}.inputs[+] = "{workspaceRoot}/jest.preset.js"
.targetDefaults.{test}.outputs[+] = "{workspaceRoot}/coverage/{projectRoot}"
```

In this representation, the use of map accessor (`{}`) for `build` and `test`
unequivocally designates them as user-defined entities, as opposed to other
properties that are consistent with the schema's definitions.

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

The specification of MERC is segregated into 3 parts:

1. Syntax
2. Semantics
3. Formatter

This segregation is to ease implementation, the syntax specification corresponds to the parser implementation, semantics the evaluator, and formatter the formatter.

# Specification (Syntax)

## Overview

The syntax of MERC will be described using ISO-14977 EBNF.

## Whitespace Insensitivity

Whitespaces are insignificant in MERC, they can be inserted between any pair of tokens, except for comments, where the newline character serves as the end of a comment.

For example, the following MERC is valid despite the irregular spacing.

```bash
.x . y {   z } =    123  .name =3
```

The syntax of whitespace is:

```ebnf
whitespace = space | newline | tab ;
newline = ? Linefeed (0x0A) ? | ? Carriage Return Linefeed (0x0D 0x0A)  ?
space = ? Space (0x20) ?
tab = ? 0x09 ?
```

## File

A MERC configuration file is structured as a list of entries. Each entry represents a configuration directive and is composed of a path and a value.

The syntax of a MERC file is:

```ebnf
file = entry, {entry};
```

## Entry

Each entry in a MERC file specifies a particular configuration setting. An entry has two components:

- **Path**: A sequence of accessors that defines where in the configuration hierarchy the value should be placed.
- **Value**: A scalar value that will be applied at the specified path.

An entry may also be preceded by a comment, or a group of comments.

Syntax:

```abnf
entry = {comment}, path, "=", value;
```

The equal sign must be the ASCII equal sign (`=`).

## Path

Syntax:

```ebnf
path = accessor, {accessor};
```

Example of paths (separated by newlines):

```python
.foo{bar}[+]
{spam}.baz[qux]
[cheese] [good] .brand .origin
```

## Accessor

There are 4 kinds of accessor:

1. Object accessor
2. Map accessor
3. Implicit array accessor
4. Explicit array accessor

The syntax of Accessor is:

```ebnf
accessor = object_accessor | map_accessor | implicit_array_accessor | explicit_array_accessor;
```

### Identifier

Before we talk about the syntax of these accessors, we need to define _identifier_.

An identifier can be either quoted or unquoted.

Unquoted identifiers must consist of only ASCII alphanumeric characters, ASCII dash (`-`) or ASCII underscore (`_`). For example, `i-am_valid999` is a valid unquoted identifier, while `hello:world` is not.

Also, unlike common programming languages, it is not necessary for a unquoted identifier to start with ASCII alphabets or underscore, it can also begins with dashes or ASCII digits. For instance, `0` and `-im_am_negative` are both valid unquoted identifiers.

Quoted identifier is identical to JSON string, which begins and ends with double quotes (`"`), and the characters in between might contain escaped characters such as `\n` or `\u1234`. For instance, `"I have whitespaces"` is a valid quoted identifier.

Syntax:

```ebnf
identifier = quoted_identifier | unquoted_identifier;
unquoted_identifier = char, {char};
char = ASCII_ALPHABETS | ASCII_DIGITS | "-" | "_";
unquoted_identifier = json_string;
json_string = '"' ,
  { ? Any unicode character except " or \ or control character ?
  | "\" ,
    ( '"' (* quotation mark *)        | "\" (* reverse solidus *)
    | "/" (* solidus *)               | "b" (* backspace *)
    | "f" (* formfeed *)              | "n" (* newline *)
    | "r" (* carriage return *)       | "t" (* horizontal tab *)
    | "u" , 4 * ? hexadeximal digit ?
    )
  } , '"' ;
```

### 1. Object accessor

Syntax:

```ebnf
object_accessor = ".", identifier;
```

Example of valid object accessors (separated by newlines):

```python
.package
."mime/type"
.0
```

### 2. Map Accessor

Syntax:

```ebnf
map_accessor = "{", identifier, "}";
```

Example of valid map accessors (separated by newlines):

```python
{react}
{"deploy to staging"}
{123}
```

### 3. Implicit Array Accessor

Syntax:

```ebnf
implicit_array_accessor = "[", "+", "]";
```

Example of valid implicit array access (separated by newlines):

```python
[+]
[ + ]
[  +]
[+  ]
```

### 4. Explicit Array Accessor

Syntax:

```ebnf
explicit_array_accessor = "[", identifier, "]";
```

Example of valid explicit array accessors (separated by newlines):

```python
[build]
["@typescript/typescript-language-server"]
[0]
```

## Values

Values refers to the part that comes after the literal `=` in an entry.

Syntax:

```ebnf
value = json_scalar | non_json_scalar;
json_scalar = json_string | json_number | json_boolean | json_null;
non_json_scalar = multiline_string;
```

### 1. JSON scalars

JSON scalars has the exact same syntax as those defined in the JSON specification.

- **Strings**: Enclosed in double quotes.
  - Example: `"Hello, World!"`
  - **Booleans**: `true` or `false`.
  - Example: `true`
- **Numbers**: Integers or decimals without quotes.
  - Integer Example: `42`
  - Decimal Example: `3.14`
- **Null** : `null`

### 2. Non-JSON scalars

These are scalars that are not present in the JSON specification.

#### Raw String

Raw strings are string where every characters are treated literally, for example, `\n` means two characters, `\` and `n`.

There are two kinds of raw strings, one surrounded by one single quote on both end, the other surrounded by three single quotes on both end.

Syntax:

```ebnf
raw_string
  = "'"  , ? Any unicode character sequence except ' and newline ?, "'"
  | "'''", ? Any unicode character sequence except ''' ?, "'''";
```

Example:

```python
# What you see is what you get
.winpath  = 'C:\Users\nodejs\templates'
.winpath2 = '\\ServerX\admin$\system32\'
.quoted   = 'Tom "Dubs" Preston-Werner'
.regex    = '<\i\c*\s*>'
```

When there are multiple lines, the characters that appear before the first newline or after the last newline should be trimmed.

For example, the following multiline-string:

```python
.x = '''This is trimmed

Not trimmed

This is also trimmed'''
```

...translates into the following JSON:

```json
{ "x": "\nNot trimmed\n" }
```

## Comments

Comments provide context or explanations for an entry and must be placed on their own line directly above the entry they describe. Comments start with a `#` symbol.

Syntax:

```ebnf
comment = "#", ? Any unicode character except newline ?;
```

Example:

```bash
# This comment describes the following setting for background color
.settings.background = "blue"
```

# Specification (Semantics)

## 1. Type of a parent path

The type of a given parent path is determined by either its direct descendant accessor (if it has one) or of the value assigned to it.

The most ancestral parent path is the root path, which is literally nothing.

In the following example, the root value type is inferred as Object because the first entry is `.x`, and `.x. implies the parent value is Object.

```python
.x{y} = 2
```

MERC allows any compound type to serve as the root value. This means that configurations can have top-level arrays, maps, and tuples.

The second example below demonstrate the type of each parent path:

```python
.foo{bar}[spam] = 2
```

- The type of root is Object, via `.foo`
- The type of `.foo` is Map, via `{bar}`
- The type of `.foo{bar}` is Array, via `[spam]`
- The type of `.foo{bar}[spam]` is Number, via `2`

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

## 4. Entry order

Entry order is only important for array elements, any other entries can be freely ordered.
To demonstrate, both of the examples below are semantically equivalent:

```python
.foo[a].x = 1
.comment = "Hello"

.foo[b].x = 3
.foo[a].y = 2


.foo[b].y = 4
```

```python
.comment = "Hello"
.foo[a].y = 2
.foo[a].x = 1

.foo[b].y = 4
.foo[b].x = 3
```

## 5. Array element order

The element order of an array is determined by the order of their first (from top to bottom) occurence.

In the following example:

```python
.[z].x = 3
.[y].x = 4
.[y].b = 2
.[z].b = 1
```

... the element indexed by `z` will come before `y` in the constructed array, because it's first occurence is `.[z].x = 3`,
which comes before the first occurence of the element indexed by `y`, which is `.[y].x =4`.

## 6. Implicit array accessor

The implicit array accessor `[+]` is used when the user does not want to explicitly define the array keys.
Every occurence of `+` should be substituted with a global unique value, which can be easily achieved by having a global unsigned integer counter.

This also implies that when implicit array accessor is used, no objects/maps with more than one key can be constructed,
for example, the following means two objects.

```python
[+].x = "hello"
[+].y = "hey"
```

The above translates into the following JSON:

```json
[{ "x": "hello" }, { "y": "hey" }]
```

## 7. Explicit array keys

Explicit array keys should not be included in the constructed value, they are solely used as labels to ease configuration navigation.

In other words, the array keys defined in MERC cannot be consumed by the application code.

## 8. Object and Map

Object and Map are identical under the hood.

## 9. Case-sensitivity

MERC is case-sensitive, for example, `.x` and `.X` are different paths.

# Specification (Formatter)

Every compliant MERC formatter must adheres to the following rules to ensure consistency and readability.

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

Within maps and objects, keys must be sorted ascendingly, ignoring quotes for quoted keys.

Sorting works by first escaping all non-ASCII characters into ASCII in the format of `\uNNNN`,
then the keys are compared with each other lexicographically.

The escape is necessary because there's is no deterministic lexicographical order for Unicode characters,
due to different cultures.

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
For example, suppose the original config looks like this:

```python
.fruits[+] = "apple"

.fruits[+] = "banana"

```

- Correct:

```python
.fruits[+] = "apple"
.fruits[+] = "banana"
```

- Incorrect:

```python
.fruits[+] = "banana"
.fruits[+] = "apple"
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

## 8. **String Formatting**

Format strings as raw string whenever possible if the content contains at least one newline character,
and the content does not contain `'''`.

Example:

- Correct:
  ```python
  .greeting = '''
  Hello,
  World!
  '''
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
| File Extension | `.merc`            |
| MIME type      | `application/merc` |
| Encoding       | UTF-8              |
