# Candela

A collection of quality of life utilities that help me at my job.

## Flattener

Feature straight out of [delflat](https://github.com/eriizu/delflat). The goal
is to flatten a directory structure. That's handy for teachers that use MOSS,
because it considers different folders as different students.

Example source:
- student_a/
  - src/
    - main.c
    - has_opt.c
    - str/
      - strdup.c
- student_b/
  - src/
    - main.c
    - check_args.c
- student_c/
  - src/
    - main.c
    - has_opt_value.c
    - str/
      - strdup.c

```sh
candela flatten --source=./src --dest=./dest $(find -name *.c")
```

Produces dest folder containing:
- student_a/
  - main.c
  - has_opt.c
  - strdup.c
- student_b/
  - main.c
  - check_args.c
- student_c/
  - main.c
  - has_opt_value.c
  - strdup.c

If there are filename collisions, rerun with `-k` to keep path components inside
the file names: `src/str/strdup.c` -> `src.str.strdup.c`.

> [!warning]
> Paths aren't canonicalized yet

## Cleanner

The idea for this feature is born out of a simple enough problem: my works has me cloning
a lot of repositories, install their dependancies, compile their source; and
I don't always remmember to clean them right away.

Going arround my filesystem, using `fd`, `yarn cache clean`, `cargo clean`,
`make fclean`, `rm random_objet.o`, gets old very quickly. It sound like the perfect
job for a script or a program.

Hence this feature. It does some of these tasks automatically. It can handle:
- rust projects;
- yarn projects (with corepack enabled);
- npm projects;
- C/C++ projects, though I don't trust all the makefiles I use for cleaning,
  hence the current fallback that looks for `.o` files and asks to delete them.

Usage:

```sh
candela clean DIRECTORY
```

The program will recursively look for project directories starting from the paths given as arguments.

Exemple:

```sh
candela clean ~/repositories ~/projects
```

