# Grim Dawn - My Important Items

## Why does this exist?

I was fed up checking which MIs I wanted to keep around every time I wanted
to clean out my inventory on the off-chance they were used in crafting recipes.

At first, I just went through and added (C) to the names of all the MIs
that are used in crafting.

I then immediately wanted to do it for items needed for builds I wanted to try.
Now I also wanted to add (B).
That list grew and became unweildy, and thus this tool was born.

## What does it do?

Allows you to specify simple replacement rules for Grim Dawn item names,
for example "Append (C) to the beginning of the name of every item in this list".

## Limitations

Since the item name and affixes, including level affixes like "Mythic" are separate
localization entries, it's impossible to specify "only \*Mythic\* Kymon's Badge".

If sub-sections of a single item in the input file matches multiple item names,
you'll get double replacements, so I hope you don't want a "Kymon's Badge of Ascendant Hood"!

## Usage

```
grim-dawn-important-items [OPTIONS] --config <CONFIG>

Options:
  -i, --input <INPUT>    Input file. Reads from stdin if set to '-'
      --config <CONFIG>  Path to the config TOML file [default: config.toml]
  -d, --diffs            Diffs Only: only output changed lines
  -h, --help             Print help
  -V, --version          Print version
```

Outputs to stdout, so use `>` to save the output to a file.

## Config file format

TOML.
You can define multiple `[[category]]` blocks.

```toml
[[category]]
label = "(C) %"
item_names = [
    "Ascendant Hood",
    "Ascendant Shoulderguards",
]

[[category]]
label = "% (B)"
item_names = [
    "Kymon's Badge",
]

```

- `label` is a replacement template. The token `%` is replaced with the matched
  item name. If label contains no `%`, the original name is dropped entirely and
  `label` is used verbatim.
- `item_names` is a list of names to match. Matching is a **substring** test
  against the part of the line **after the first `=`** (so the `tag...=` key is
  preserved untouched).

### Multiple category matches

If more than one category matches, the labels are applied in order by replacing
the original item name in the string with the new category.
If you're only doing prefixes, they appear in definition order.
If you're mixing and they all match the same item it'll appear pretty random.

Given:

```toml
[[category]]
label = "(C) %"
item_names = [
    "Kymon's Badge",
]

[[category]]
label = "(B) %"
item_names = [
    "Kymon's Badge",
]
```

the line `tagMedalB004={^L}Kymon's Badge` becomes

```
tagMedalB004={^L}(C) (B) Kymon's Badge
```
