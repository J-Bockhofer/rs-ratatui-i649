# Example: Problem B

Example for issue 649, Problem B
[Issue](https://github.com/ratatui-org/ratatui/issues/649)

## The program

1. Reads new lines added to a file (specified in app.rs, Line 55)
2. Sends new lines to the app @ home.rs (->`Action::IONotify`)
3. App parses lines into styled contents and renders a list of styled lines 

To use the program change Line 55 in app.rs to point to the text.txt included in this repo. 
Do `cargo run` and copy+paste a line in the text.txt file and save it.

## Recap of Problem B

When doing the above with this branch **it won't compile** due to inadequate lifetime specifiers.
Trying to implement explicit lifetimes into the Component trait led to the propagation of needing to declare lifetimes in app.rs etc.

Which after getting advice from @kdheepak led to the solution found here:
[Solution B](https://github.com/J-Bockhofer/rs-ratatui-i649/tree/solutionB)
