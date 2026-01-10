A quick + dirty little REPL (Read + Eval + Print Loop) command line tool to
verify if single or multi line input is valid OGC CQL2 expression, or not.

Entering the sequence of two tildas `~~` followed by `↵` (the \[ENTER\] key)
initiates a multi-line mode which ends when `Ctrl-D` is pressed. In this
mode consecutive input is concatenated into one string before processing.

The program will first attempt to parse the input as a TEXT based expression
of CQL2. If it fails, it will try again treating it as JSON. In either
cases if it succeeds it will output an intermediary representation of the
expression. On the other hand, if it fails, an error message (in
<font color="red">red</font>) will be printed to `stderr`.

To start the loop enter...
```bash
cargo run --bin repl↵
```
To exit the program, press `Ctrl-D`.
