# Motivation
Command-line tool to rename files so that managing and viewing related files is easier. In particular, when listed with `ls`, the related files should appear next to eachother. Given files `a`, `b`, and `c` as input, rename them to `a_a`, `a_b`, `a_c`.

The name prefix (sometimes called "headname" in the code) is the basename (filename minus extension) of the first file that does't look like it is already part of a sequence.

## Goals
- No complicated or fragile wrappers.
- No metadata or extra files needed.
- Scriptable. Can be invoked by shell scripts or scriptable tools.
- Aware of filename extensions (eg. when given `a.txt` and `b.txt`, rename `b.txt` to `a_b.txt` not `a.txt_b.txt`).
- Simple architecture, decouple stages where possible.

## Non-goals
- No GUI.
- No generic file management abilities (eg. list files, view files, etc.). Focus on a single task.
- No ready-made integrations with other tools. It is easier and cleaner to implement these as shell scripts that wrap this program.
- No mechnisms to "collect" files and then rename them (no state is kept between invocations).

# Usage
Give paths as input on stdin, one path per line. Paths can be either an absolute path (starting with `/`), a relative path (starting with `./`), or a filename (no slashes). Relative paths and filenames are interpreted relative to the current-working directory. Note: beware of how this interacts with invoking this program from another program - the CWD may not be what you expect. To avoid confusion, using absolute paths for everything is recommended.

Hazard alert: do not provide the same file as input twice. This may result in incorrect behavior.

"Dry-run" can be accomplished by passing `--execute-plan=false --show-plan=true`.

## Add new file to a sequence
```
$ ls
a_1.txt a_2.txt 3.txt
$ ls -1 | feeq
Rename /3.txt to a_3.txt
```

Rudimentarily detect when a mixture of sequences are input:
```
$ echo -e "a_b\nc_d\n" | target/debug/feeq --execute-plan false
Could not select prefix due to: More than one input files look like they are already part of a sequence. These are their names:
a_b
c_d
```

## Flags
```
feeq

USAGE:
    feeq [OPTIONS]

OPTIONS:
        --execute-plan <BOOLEAN>    Execute the rename plan. When false, plan is constructed and
                                    optionally printedaccording to other args, but never run.
                                    [default: true] [possible values: true, false]
    -h, --help                      Print help information
        --separator <separator>     Separator between "head" name and original name when renaming.
                                    [default: _]
        --show-plan <BOOLEAN>       Output the rename plan before performing the renames. [default:
                                    true] [possible values: true, false]
```

## Scripting tips
Build up a list of files to input, then process them all at once:
```
$ echo "a" >> seq.txt
$ echo "b" >> seq.txt
$ echo "c" >> seq.txt
$ cat seq.txt | feeq
Rename /home/blzbrg/a to /home/blzbrg/a_a
Rename /home/blzbrg/b to /home/blzbrg/a_b
Rename /home/blzbrg/c to /home/blzbrg/a_c
```
