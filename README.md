# Grim Dawn - My Important Items

## What does it do?

Allows you to specify simple replacement rules for Grim Dawn item names,
for example "Add the prefix '(C) ' to the item names in this list".

Like a lot of players, I use [Grim Raynbow](https://grimraynbow.com/).
I use this tool with the output from Grim Raynbow and test against that only.

## Why does this exist?

I was fed up checking which MIs I wanted to keep around every time I wanted
to clean out my inventory on the off-chance they were used in crafting recipes.

At first, I just went through and added (C) to the names of all the MIs
that are used in crafting.

Almost immediately, I then wanted to do it for items needed for builds I
wanted to try, which meant that I also wanted to add (B).

That list grew, became unweildy, and I started to worry about updates and
having to re-apply everything.

Thus, this tool was born.

## Limitations

Since the item name and affixes, including level affixes like "Mythic" are separate
localization entries, it's impossible to specify "only \*Mythic\* Kymon's Badge".

Items that have colour information in the MIDDLE of the name, e.g. "Kymon's{^L} Badge",
aren't supported with `Custom` and only support `Prefix` and `Suffix` styles.
The outputs from Grim Raynbow do not have items named like this,
so this will only be a problem for highly customised setups.

If you want a label to inherit the colour of the item you can either use `Suffix` or
the `Custom` style. You can use "(C) %" to emulate `Prefix` with the `Custom` style.
Remember that `Custom` style does not support interspersed colours in the item name.

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

All outputs go to stdout, so use `>` to save the output to a file:

```bash
grim-dawn-important-items --input tagsgdx3_uimain.txt > tagsgdx3_uimain-mine.txt
```

If you want to perform an in-place replacement, use the following:

```bash
grim-dawn-important-items --input tagsgdx3_uimain.txt > tagsgdx3_uimain.out && mv tagsgdx3_uimain.out tagsgdx3_uimain.txt
```

To run directly from source, replace `grim-dawn-important-items` above with `cargo run --release --`:

```bash
cargo run --release -- --input tagsgdx3_uimain.txt > tagsgdx3_uimain-mine.txt
```

The input is streamed, so you can't directly overwrite the input file.
I'll probably change this eventually, because the language files aren't
actually long enough to warrant streaming.

## Config file format

Use TOML formatting.

You can define multiple `[[category]]` blocks.

```toml
[[category]]
label = "(C) "
style = "Prefix"
item_names = [
    "Ascendant Hood",
    "Ascendant Shoulderguards",
]

[[category]]
label = " (B)"
style = "Suffix"
item_names = [
    "Kymon's Badge",
]

[[category]]
label = "[%]"
style = "Custom"
item_names = [
    "Ellena's Necklace",
]
```

- The combination of `label` and `style` determines what gets inserted.
    - In `Prefix` style, the label is inserted at the start of the item name,
      **before** colour information, so it does not inherit the item's colour.
    - In `Suffix` style, the label is inserted at the end of the item name,
      which is **after** colour information, so it inherits the item's colour.
    - In `Custom` style, the token `%` is replaced with the matched item name.
      If `label` contains no `%`, the original name is dropped entirely and
      `label` is used verbatim.
- `item_names` is a list of names to match. The part of the line **after the first `=`**
  is stripped of meta-information, and then compared to the item name specified
  here.
    - Meta-information means is anything inside `{}` (colour information) and
    anything inside `()` (Grim Raynbow marks set items with `(S)`)


### Config Tips

If you set a colour via `label`, I recommend terminating it with `{^-}` to reset the colour.

Grim Raynbow does not explicitly colour (all?) Epic (Blue) items,
so a `Prefix` can leak any colour you set into the item name.

Don't do this:
```toml
label = "{^R}(C) "
```

Do this instead:
```toml
label = "{^R}(C) {^-}"
```

#### Awakenable Epic items don't work

Grim Raynbow omits Awakenable Epic items in the default config.
You can confirm this with `--mode validate`: it will log all
Awakenable Epic items if you use Grim Raynbow's default settings.

On the Grim Raynbow site, under Modules::Specials you'll find Block::Awakenable.
Set this to `Blue {^B}` to force it to include Awakenable Epics in the ouput.
Confirm with `--mode validate` that items are no longer logged.

### Multiple category matches

If more than one category matches, the labels are applied in order by replacing
the original item name in the string with the new category.
If you're only doing `Prefix`es, they appear in the opposite to definition order.
If you're mixing and they all match the same item it will appear pretty random,
but it is consistent.

Given:

```toml
[[category]]
label = "(C) "
style = "Prefix"
item_names = [
    "Kymon's Badge",
]

[[category]]
label = "(B) "
style = "Prefix"
item_names = [
    "Kymon's Badge",
]
```

the line `tagMedalB004={^L}Kymon's Badge` becomes

```
tagMedalB004=(B) (C) {^L}Kymon's Badge
```
