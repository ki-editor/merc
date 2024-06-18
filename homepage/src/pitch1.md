# A taste of MARC

In MARC, the path of every value is explicitly written down, redundantly.

```bash
# Object
.user.name = "Lee"
.user.isVerified = true

# Map
.dependencies{react} = "^0.1.0"
.dependencies{graphql} = "1.2.3"

# Array
.excludes[i] = "dist/"
.excludes[i] = "node_modules/"

# Tuple
.position(i) = 45.6
.position(i) = -985.12

# Array of objects
.jobs[i].name = "build"

# Use `[ ]` to set value to the last array element
.jobs[ ].only = "main"
```

---

# Why MARC?

## 1. Simplified Copy-Paste Operations

Consider the following YAML snippet from the GitHub Actions documentation:

```yaml
on:
  push:
    paths:
      - "**.js"
```

Directly copying and pasting this snippet into an existing GitHub Actions configuration file won't work as expected.

Instead of a simple paste, you're required to perform a **merge**—a process that's both tedious and prone to errors, especially as the length of the configuration grows.

In this scenario, you should only copy `paths:\n      - '**.js'` and insert it under the `push` field, not the entire snippet.

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

If GitHub Actions configurations were authored in MARC, the documentation example would be transformed into the following format, making copy-pasting effortless because **no merging is necessary**:

```bash
on.push.paths[i] = "**.js"
```

## 2. Streamlined Change Review

The challenge of reviewing changes is amplified with large configuration files. Consider the JSON diff below; the necessity to scroll up and down makes it difficult to confidently identify which path in the config has been altered:

```diff
    "type": "spot",
-   "size": "t3-medium",
+   "size": "t3-large",
    "securityGroup": "internal"
```

MARC simplifies this process. Changes are immediately apparent, eliminating any guesswork:

```diff
    machines{api}.type = "spot"
-   machines{api}.size = "t3-medium"
+   machines{api}.size = "t3-large"
    machines{api}.securityGroup" = "internal"
```

## 3. Easily Decipherable

When attempting to comprehend a configuration file, it’s often necessary to mentally decode it into a format akin to MARC to fully understand its structure.

Creating configurations in MARC conserves cognitive energy for troubleshooting real issues, eliminating any uncertainty about the accuracy of your interpretation.

## 4. Semantic Clarity

In contrast to many common configuration formats, MARC clearly differentiates between Objects and Maps in its syntax.

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

MARC eliminates such ambiguities:

```python
.targetDefaults.{build}.cache = true
.targetDefaults.{build}.dependsOn[i] = "^build"
.targetDefaults.{build}.inputs[i] = "production"
.targetDefaults.{build}.inputs[i] = "^production"
.targetDefaults.{build}.options.main = "{projectRoot}/src/index.ts"
.targetDefaults.{test}.cache = true
.targetDefaults.{test}.executor = "@nx/jest:jest"
.targetDefaults.{test}.inputs[i] = "default"
.targetDefaults.{test}.inputs[i] = "^production"
.targetDefaults.{test}.inputs[i] = "{workspaceRoot}/jest.preset.js"
.targetDefaults.{test}.outputs[i] = "{workspaceRoot}/coverage/{projectRoot}"
```

With MARC, every element’s role is transparent. User-defined properties are clearly marked, eliminating any guesswork and streamlining both documentation and maintenance processes.

# Playground

Discover the functionality of MARC with the following 4-way converter.  
Edit any format—MARC, JSON, YAML, or TOML—and see immediate updates in the others.
