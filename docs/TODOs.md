# Repeater Atlas TODOs

These sections are items to be fixed in Repeater Atlas.

_Rules_:

- Each section heading is the issue of the title and then the body will describe
  whatever is needed to be done.
- If an issue is closed, the heading will be ~~stricken through~~.
- Issues may be linked and related to each other.

## RA-5: Use fuller types for frequencies

See RA-4, do the same for frequencies.

- The frequency format is ##0.000# (Hz|kHz|MHz|GHz).
- The value indicates the suffix
- Always show three digits, but show more if there are more digits.
- Write unit tests.

## RA-4: Use fuller types for Maidenhead locators

Instead of string types, make a MaidenheadLocator type that wraps a string. Use
the maidenhead crate when constructing values.

The default display of this type should be used in the samples to format the
value.

## RA-3: Support repeater linking

Some repeaters are only for linking to other repeaters.

This issue must first be properly designed before implemented.

## RA-2: Add support for all enum variants of Repeater Service

After the merging of all the Repeater Service tables it is easier to just
iterate over all services and render them properly.

## RA-1: Improve map on repeater details page

Right now the map is a box containing all the closed repeaters, but it should be
centered around the current repeater.
