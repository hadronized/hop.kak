# Hop: hinting brought to Kakoune selections

<p align="center">
  <img src="https://github.com/phaazon/hop.kak/assets/506592/6afd368b-5d42-4a70-af10-f55a9c9c366b"/>
</p>

Table of content:

- [Install](#install)
- [Configuration](#configuration)
  - [Kakoune options](#kakoune-options)
  - [`hop-kak` options](#hop-kak-options)
- [Usage](#usage)
- [Workflow examples](#workflow-examples)
  - [Default keyset](#default-keyset)
  - [Better selections](#better-selections)
  - [Select and hint visible words](#select-and-hint-visible-words)

This binary is intended to be used with the [Kakoune editor](https://kakoune.org/), and provides _hinting_ capabilities
based on the current selections of the user. The workflow is simple:

1. Make a selection in your buffer.
2. Call the binary by providing the selections in `%sh{}` block — via `$kak_selections_desc`, typically.
3. Hints appear. You can press the keys in order of each hint to reduce the hints.
4. You typically reduce until only one hint remains; in such case, hinting is disabled and you are left with the sole
  selection. However, you are free to leave hinting at any reduction step by pressing the `<esc>` key.

## Install

Currently, the only installation channel is https://crates.io. It requires [`cargo` to be installed](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```bash
cargo install hop-kak
```

## Configuration

### Kakoune options

You must include the [hop.kak](./hop.kak) file before trying to use Hop. It contains:

- The `hop_ranges` option. Used to highlight your buffer with the labels.
- The `hop_label` face definition. Feel free to override the default.

### `hop-kak` options

`hop-kak` — the built binary — doesn’t have any configuration file. Instead, it is configured by passing CLI arguments:

- `-k --keyset`: the keyset to use. This depends on your keyboard layout. Choose it wisely! It must not have any
  duplicate key, and keys are ordered by importance; i.e. the keys that are easier to reach should appear first.
  - For QWERTY, we recommend `TODO`.
  - For AZERTY, we recommend `TODO`.
  - For BÉPO, we recommend `etisura,cnovpdélxqygàhfbjz`.
- `-s --sels`: selections to hint. You should always pass `$kak_selections_desc` here.
- `-l --labels`: previous generated labels. You should never need to use that argument.
- `-z --key`: key for reduction. You should never need to use that argument.

## Usage

The binary was made with few responsibilities, so that people can use it in a wider variety of situations. For this
reason, you will have to build a bit around `hop-kak`. `hop-kak` works by reading selections in, highlighting them and
making Kakoune wait for a key press to either abort, or reduce the hints. If you decide to reduce the hints, `hop-kak`
will filter your selections and reduce them to map the new set of hints. Hence, `hop-kak` can basically be seen as a
trie reducer for Kakoune selections. It is then very composable. You pass it initial selections, and it interactively
filters them.

Whatever your selections, you will always want to start a hopping session with the following command:

```kak
eval -no-hooks -- %sh{ hop-kak --keyset "<YOUR_KEYSET_HERE>" --sels "$kak_selections_desc" }
```

For instance, with the bépo keyboard layout, you could map the `è` key to start hopping with your current selections:

```kak
map global normal è ':eval -no-hooks -- %sh{ hop-kak --keyset "etisura,cnovpdélxqygàhfbjz" --sels "$kak_selections_desc" }<ret>'
```

Then, it’s up to you to come up with your own workflow!

## Workflow examples

### Default keyset

You should have an option to set your keyset if you intend on having several workflows. For instance, for bépo:

```kakoune
declare-option str hop_kak_keyset 'etisura,cnovpdélxqygàhfbjz'
```

### Better selections

Something that is pretty useful is to map a key to select the visible part of the buffer. `<a-%>` is a good candidate,
as it’s not mapped by Kakoune for now:

```kakoune
map global normal <a-%> ':execute-keys gtGbx<ret>'
```

This will help with creating selections. We assume you have this binding below. You can also make a command for that:

```kak
define-command hop-kak %{
  eval -no-hooks -- %sh{ hop-kak --keyset "$kak_opt_hop_kak_keyset" --sels "$kak_selections_desc" }
}
```

### Select and hint visible words

With the `<a-%>` mapping, you can _select_ words with `<a-%>s\w+`, and then press your mapping to start hopping around.

A slightly better approach to reduce the number of keys and control keys to type is to create a small function
like this:

```kak
define-command -override hop-kak-words %{
  exec 'gtGbxs\w+<ret>:eval -no-hooks -- %sh{ hop-kak --keyset "$kak_opt_hop_kak_keyset" --sels "$kak_selections_desc" }<ret>'
}
```

And mapping it to your key; e.g. `SPC è`:

```kak
map global user è :hop-kak-words<ret>
```
