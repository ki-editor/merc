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

MARC eliminates this complexity by using a path-value syntax that allows for direct insertion without concern for structural conflicts:

```bash
on.push.paths[i] = "**.js"
```

With MARC, you can effortlessly copy and paste without needing to merge—**no complex integration required**. This simplicity minimizes errors and streamlines configuration updates.

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

## 4. Natural Documentation

Documenting configuration schemas in hierarchical formats like JSON or YAML often lacks clarity and can be unintuitive. Ironically, most existing configuration documentations gravitate towards a MARC-like syntax for maximum clarity, as seen in these examples:

| Documentation                                                                                                           | MARC-like Syntax Example           |
| ----------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| [Gitlab CI/CD Yaml](https://docs.gitlab.com/ee/ci/yaml)                                                                 | `cache:key:files`                  |
| [Github Actions Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions) | `jobs.<job_id>.defaults.run.shell` |

This MARC-like syntax, while clear, often differs significantly from the actual configuration format, creating a disconnect that readers must bridge to fully grasp the function of each property. Additionally, documentation writers must invest considerable time and effort to create and maintain this pseudo MARC-like syntax, which is not required in the actual configuration files.

MARC, on the other hand, eliminates this disparity and extra effort. The syntax used in documentation is identical to that used in actual configurations—what you see is what you get. This one-to-one correspondence between documentation and implementation simplifies understanding and reduces the cognitive load.

## 5. Straightforward Communication

Navigating configuration errors in hierarchical formats can be as perplexing as giving directions in a labyrinth. It often involves convoluted explanations akin to guiding a tourist through a maze of streets: _"Proceed straight, then take a left at the souvenir shop, followed by a right turn…"_

In contrast, MARC’s simplicity turns these complex instructions into straightforward directions. It’s like pointing directly to the destination and saying, _"The restroom is right there."_

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

If the configuration was in MARC instead of XML, the answer would look like this, without the navigational instructions:

> I solved my problem by adding the following lines:
>
> ```python
> configuration.system.webServer.httpErrors.remove[i].statusCode = 404
> configuration.system.webServer.httpErrors.remove[ ].subStatusCode = -1
> ```

The same goes for communicating modifications or deletions of existing values in the configuration file.

## 6. Semantic Clarity

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

In this representation, the use of map accessor (`{}`) for `build` and `test`
unequivocally designates them as user-defined entities, as opposed to other
properties that are consistent with the schema's definitions.

# Playground

Discover the functionality of MARC with the following 4-way converter.  
Edit any format—MARC, JSON, YAML, or TOML—and see immediate updates in the others.
