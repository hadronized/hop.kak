# Design doc about hinting

_Hinting_ is the process of displaying _labels_ atop of a buffer (i.e. window) to allow faster operations. Especially,
applied to a text editor, hinting is often wanted to be able to _move_ faster. Here, faster means _less key strokes_.
This document describes the design choices made with `hop.kak`.

## Hinting should require a minimal mental effort

Whenever we want to jump to a given position in a buffer, we should not leave the place where we want to jump to. Many
approaches exist where you need to use _relative numbers_ to first jump to the line where your jump target is, and then
move horizontally. That requires too much effort as you need to know which line to jump to (people without line numbers
will have a pretty bad experience here), and how to move efficiently horizontally.

To solve that problem, `hop.kak` generates labels _right at the place you want to jump_, and you type the labels you
see. There is no need to think in terms of line numbers and horizontal movements.

## Hinting is visual, not semantics

Hinting is about what _we see_, not what _things are_. For instance, if you look at some place in a buffer you want to
jump to, you should only have to think about _“wanting to jump there”_. Wondering whether that position is part of a
function argument, a comment or an operator is a waste of mental effort and energy.

We must think of hinting a bit as if we were pointing our finger at the screen and say _“Go there!”_. There is no
semantics interpretation of _there_.

## Hinting should be fast

It’s an obvious one, but we want to hint a buffer to move fast. We must generate and display the hints in a couple
milliseconds maximum, to prevent disrupting coding flow.

## Labels should optimize length

A big design decision was about how to generate the labels. The concept of _keyset_ allows users to customize the
distribution: left-most keys in the keyset are _more important_ (meaning that we want to have them first). Built atop
keysets, labels are generated in a dynamic way (using a mutable trie), by gradually growing the trie and exhausting
the keyset. Once we are running out of key to use, we add another layer atop the last key of the keyset. For instance,
with the keyset `abcd`, if we have 4 words to hints, we will generate the following labels:

```
a
b
c
d
```

However, if we ask to hint 5 words, we don’t have enough keys in the keyset. Hence, we create a layer on `d`:

```
a
b
c
da
db
```

The great thing about the design is that the keys that are more important are more frequent: on 7 chars, only 2 contain
`d`, which is the worst rank, and only 1 contains `c`; on the other side, `a` and `b`, the top-2 keys, are present on 4
characters in the distribution.

The other great aspect of this approach is that even with many items to hints, and a big-enough keyset, we might still
yield 1-char label, and very very rarely more than 2-char labels, which is ideal: it means less key strokes.

## Interactivity is important

Once the labels are generated, we want the editor to present some form of interactivity as we type the keys. In the
example above, typing `d` means that we either want to go to `da` or `db`. We can then completely remove the labels for
`a`, `b` and `c`, since they do not start with `d`.

This kind of interaction might look distracting at first, but on big buffers with many hints, it prevents having the
feeling that we are not making any progress while reducing the hints. It provides a direction, a focus, which aligns
with where we are looking at.

## Labels should have a head and a tail

A label length is comprised between 1 and 2, rarely 3 and very rarely 4, if the keyset is big enough — in theory, there
are no limit, but hitting 3-char labels already requires a lot of things to hint if your keyset contains a good amount
of keys. When a label is displayed, if it has more than 1 key (for instance 2), then it is important to make it clear
which key of it we should type to reduce it. There are two reasons for making the head (first character) and the tail
(the rest) of different colors:

- It makes it possible to tell two adjacent label apart. For instance, if you have labels `da` and `db` close to each
  other, without two colors, one for the head and one for the tail, `dadb` would look like the same label.
- Sometimes, a label is too big for the selection it maps to. For instance, if we have `da` over the `a` word, we
  cannot display the full label over `a`. We decide to then just display the head, and because it has a different color,
  if we have two adjacent heads together, we know that they correspond to different labels. For instance, if we have
  labels `da`, `db` and `dc` over dots (`.`), and that you have `...` hinted, it will first appear as `ddd`. Pressing
  `d` one will reveal `abc`.

## Hinting should only be about zipping labels to selections and handling reducing

Hinting is only about generating the labels using the trie mentioned earlier, and reducing the generated zipped
selections+labels. The selections should be provided by the user. The reason for this is that Kakoune is a superbe
tool, and multiselections are one of the best feature of the editor. It is then very composable to accept selections
as primitive construct. Users can build their own workflows by providing their own selections, and the entry barrier
is pretty low (especially with good mentoring; see the README).

Thus, there is no need to support any _modes_. No word mode. No line mode. No tree-sitter mode. All those modes have to
be provided as selections by users. That design helps with making a super small tool, very composable and easy to
think about.

`hop.kak` is an interactive selection filter / reducer.
