# To Do

## Core
  [X] **Implement parsers (raw deserializers?) for suggestions**
  [X] Make it a library
  [X] Aggregate suggestions (history + web)
  [X] Re-think error handling
  [X] ~~List suggestions parsers~~
  [X] ~~Make it a daemon~~

## CLI
  [ ] Make flags enum
  [ ] Better error reporting
  [ ] Make completions file dynamic

## Prompt
  [ ] Cater for `target word "  word2   "`
  [X] Support unicode
  [ ] Add completion support
  [ ] Add suggestion support
  [ ] Support lines longer than screen width

## UI
  [ ] Make UI

## Util
  [X] Reorganize
```
vai
 |_ core (lib)
 |_ cli
 |_ ui
 |_ macros
```
  [ ] Make macro lib for enum options
