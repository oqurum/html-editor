# html-editor
A Book Visual Editor for editing text Eg. Highlight, Underline, Draw, etc.

When registering the listener it will cache the Text Nodes inside the HTML Element. Which means this library only works with a static webpage after registered.

## Todo:
 - Determine if I should include Italicize, Bold.
 - Drawing
 - Notes
 - Referencing
 - Anchoring (eg. # in HTML)
 - and more.


# Running/Building

To run and build the application you need to do the following:

[Install Trunk](https://trunkrs.dev/#install). It's used for building the frontend.



To build the example:
```bash
cd examples/full
trunk build
```

To build and serve the example website:
```bash
cd examples/full
trunk serve
```