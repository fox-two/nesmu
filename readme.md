# Nesmu
Simple NES emulator written in rust with minimal dependencies.

Has support for nrom games and partial support for mmc3 games.
While not very accurate, can run some games like super mario bros 1 or super mario bros 3.

## Running
Just pass a .nes file:
```
./nesmu smb3.nes
```

The controls cannot be configured and have the following keybinds:


 Button        | Mapped to
 --------------|-------------
 Start         | Enter
 Select        | Backspace
 A             | A
 B             | D
 Up            | Up Arrow
 Down          | Down Arrow
 Left          | Left Arrow
 Right         | Right Arrow