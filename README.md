# Hop: hinting brought to Kakoune selections

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

### `hop` options

`hop` — the built binary — doesn’t have any configuration file. Instead, it is configured by passing CLI arguments:

- `-k --keyset`: the keyset to use. This depends on your keyboard layout. Choose it wisely! It must not have any
  duplicate key, and keys are ordered by importance; i.e. the keys that are easier to reach should appear first.
  - For QWERTY, we recommend `TODO`.
  - For AZERTY, we recommend `TODO`.
  - For BÉPO, we recommend `etisura,cnovpdélxqygàhfbjz`.
- `-s --sels`: selections to hint. You should always pass `$kak_selections_desc` here.
- `-l --labels`: previous generated labels. You should never need to use that argument.
- `-z --key`: key for reduction. You should never need to use that argument.
