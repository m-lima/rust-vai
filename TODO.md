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
  [ ] ***Atty not working on windows***

## Prompt
  [ ] Cater for `target word "  word2   "`
  [X] Support unicode
  [ ] Add completion support
  [X] Add suggestion support
  [X] Support lines longer than screen width
  [ ] Review performance of wide inputs
  [ ] Hard to tell if there is a suggestion on wide input

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
  [X] ~~Make macro lib for enum options~~
