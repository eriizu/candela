# Project Cleaner

## Description

The idea for this project is born out of a simple enough problem: my works has me cloning
a lot of repositories, install their dependancies, compile their source; and
I don't always remmember to clean them right away.

Going arround my filesystem, using `fd`, `yarn cache clean`, `cargo clean`,
`make fclean`, `rm random_objet.o`, gets old very quickly. It sound like the perfect
job for a script or a program.

Hence this project. It does some of these tasks automatically. It can handle:
- rust projects;
- yarn projects (with corepack enabled);
- npm projects;
- C/C++ projects, though I don't trust all the makefiles I use for cleaning,
  hence the current fallback that looks for `.o` files and asks to delete them.

## Usage

```sh
projclean DIRECTORY
```

The program will recursively look for project directories starting from the paths given as arguments.

Exemple:

```sh
projclean ~/repositories ~/projects
```

