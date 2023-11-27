# Example: Problem A

Example for issue 649, Problem A
[Issue](https://github.com/ratatui-org/ratatui/issues/649)

## The program

1. Reads new lines added to a file (specified in app.rs, Line 55)
2. Sends new lines to the app @ home.rs (->`Action::IONotify`)
3. App parses lines into styled contents and renders a list of styled lines 

To use the program change Line 55 in app.rs to point to the text.txt included in this repo. 
Do `cargo run` and copy+paste a line in the text.txt file and save it.

## Recap of Problem A

When doing the above with this branch you can watch the CPU usage increase with each line added.
This problem mostly boils down to really inefficient placement of the regex constructor being used to color a word in a given string.
Which led to me looking in all the wrong for the issue at first.

This emphasizes however, that in parsing large lists one may want to precalculate the styles when a new string is received from the file watcher.
This brings us to Problem B [B](https://github.com/J-Bockhofer/rs-ratatui-i649/tree/problemB)
