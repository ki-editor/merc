# Semantic Rules

## Rule 1: Duplicated assignment is not allowed

Once a path has a defined scalar value, it cannot be redefined again.

```bash
.x.y = 1
.x.y = 2 # Error: Duplicated assignment
```

## Rule 2: Type of value cannot be changed once defined

In the following example, the second line is erroneous because `.x` was inferred as `Object` due to `.y`, but the second line treats `.x` as `Map` via `{z}`.

```bash
.x.y = 1
.x{z} = 2 # Error
```

## Rule 3: Array element must be initialized before being modified

The following example is erroneous because `.x` is not initialized with an element.

```bash
.x[ ].y = 123 # Error
```

To fix this, replace `[ ]` with `[i]`.

---

# Cons of MARC

- Maximally Redundant.
- Super verbose and lengthy.
- Slower to parse than other formats.
