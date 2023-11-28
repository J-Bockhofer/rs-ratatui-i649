# Example: Solution to Problem B

Example for issue 649, Solution B
[Issue](https://github.com/ratatui-org/ratatui/issues/649)

## The program

1. Reads new lines added to a file (specified in app.rs, Line 55)
2. Sends new lines to the app @ home.rs (->`Action::IONotify`)
3. App parses lines into styled contents and renders a list of styled lines 

To use the program change Line 55 in app.rs to point to the text.txt included in this repo. 
Do `cargo run` and copy+paste a line in the text.txt file and save it.

## Recap of Solution B

When doing the above with this branch _it's all fine_.
On receiving a new string the `Home.highlightio()` function will parse the stored list as before, only now it will save precalculated styled lines as a vector of this struct:

```rust

#[derive(Default, Clone)]
struct StyledLine {
  words: Vec<(String, Style)>,
}

#[derive(Default)]
pub struct Home <'a>{
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  iostreamed: StatefulList<String>,
  precalc_list: Vec<StyledLine>,
}

```

This works well enough to keep the inefficient regex around, thus solving the problem for the moment.
There are of course major improvements possible in the highlighting function still. 
Like not iterating over the entire list on each received String by pushing a new styled String into either the `Vec<StyledLine>` 
or directly storing it in the `StatefulList` to avoid duplication.
This example is kept simple and naive to highlight the problems while not changing too much between steps.


## Further improvements

Another thing that can be done with this approach is construct a stored configurable theme, like so:


```rust


#[derive(Default)]
pub struct WordStylePair {
    pub word: String,
    pub style: Style,
}

impl WordStylePair {
    pub fn new(word: String, style: Style) -> Self {
        WordStylePair { word, style }
    }
}


#[derive(Default)]
pub struct WordStyleMap {
    pub word_styles: Vec<WordStylePair>,
}

impl WordStyleMap {
    pub fn new(word_styles: Vec<WordStylePair>) -> Self {
        WordStyleMap { word_styles}
    }
    pub fn word_in_map(&self, word:String) -> bool {
        let mut res = false;
        for item in self.word_styles.iter() {
            if word == item.word {
                res = true;
                return res;
            }
        }
        res
    }

    /// return the style for a given word if found in WordStyleMap
    pub fn get_style_or_default(&self, word:String) -> Style {
        
        for item in self.word_styles.iter() {
            if word == item.word {
                return item.style;
            }
        }
        Style::default()
    }
}



pub struct Theme {
    pub word_style_map: WordStyleMap,
}

impl Theme {
    pub fn new(word_style_map: WordStyleMap) -> Self {
        Theme { word_style_map}
    }
    }

// Example for setting up a default theme
impl Default for Theme {
    fn default() -> Self {
        Theme { word_style_map: WordStyleMap{ word_styles: vec![
                            WordStylePair::new(String::from("Found"), Style::default().fg(Color::LightCyan)),
                            WordStylePair::new(String::from("Ban"), Style::default().fg(Color::LightYellow)),
                            WordStylePair::new(String::from("INFO"), Style::default().fg(Color::LightCyan)),
                            WordStylePair::new(String::from("WARNING"), Style::default().fg(Color::Yellow)),
                            WordStylePair::new(String::from("NOTICE"), Style::default().fg(Color::LightGreen)),
                        ]}}
    }
}

```

This can of course further be improved on by implementing a hashmap for style lookup when working with more complex themeing / more keywords.

Given a theme as above and extending it a little, the function for styling incoming message in this case could look like this:

```rust

// in Home
{
    stored_styled_iostreamed: StatefulList<StyledLine>, 
}

// self is a reference to Home in this case
  pub fn style_incoming_message(&mut self, msg: String) {

    let collected: Vec<&str> = msg.split("++++").collect(); // new line delimiter in received lines, if more than one got added simultaneously

    for tmp_line in collected {
      if tmp_line.is_empty() {
        continue;
      }
      let mut thisline: StyledLine = StyledLine::default();
      let words: Vec<&str> = tmp_line.split(" ").collect();
      let mut held_unstyled_words: Vec<&str> = vec![];
      for word in words.clone(){
        // get style for word
        let mut word_style = self.apptheme.word_style_map.get_style_or_default(word.to_string()); // Detector for constant word
        if word_style == Style::default() {
          // try regex styling on word
          word_style = self.apptheme.regex_style_map.get_style_or_default(word.to_string()); // Detector for regex
        }
        

        if word_style == Style::default() {
          // If no detector has returned any styling
          held_unstyled_words.push(word);
        }
        else {
          // word is styled
          // if there are any held words push them with default style and reset held words
          if held_unstyled_words.len() > 0 {

            thisline.words.push((held_unstyled_words.join(" "), self.apptheme.default_text_style));
            held_unstyled_words = vec![];
          }
          // push styled word with space in front - TODO word is in first position and does not need a whitespace prefixed
          thisline.words.push((format!(" {}", word.to_string()), word_style));

        }
        // terminate
        if &word == words.last().unwrap() {
          thisline.words.push((format!(" {}",held_unstyled_words.join(" ")), self.apptheme.default_text_style));
        }

      }

      self.stored_styled_iostreamed.items.push(thisline);
      self.stored_styled_iostreamed.trim_to_length(20);
    }// end per line
  }

```

This accomplishes a number of things:

1. Leaves the render function untouched by any string parsing
2. Tucks away the definitions of regexes to occur only once on startup
3. Avoids duplication of the list as freshly received lines are directly pushed to the `StatefulList<StyledLine>`
4. Contains the styles needed for word highlighting in a central location that is not bound by lifetimes

For reference here is the parsing occuring in the render/draw function with this approach

```rust
fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {

      let iolines: Vec<ListItem> = self
        .stored_styled_iostreamed
        .items 
        .iter()
        .map(|i| {

          let mut line: Line = Line::default();
          for word in i.0.words.clone() {
            let cspan = Span::styled(word.0, word.1); 
            line.spans.push(cspan);
          }

          ListItem::new(line)
        })
        .collect();

      let iolist = List::new( iolines)
          .block(Block::default()
            .borders(Borders::ALL)
            .border_style(self.apptheme.border_style)
            .title("iolist_title")
          )
          .highlight_style(self.apptheme.highlight_style)
          .highlight_symbol(">> ");        

    f.render_stateful_widget(iolist, layout[1], &mut self.stored_styled_iostreamed.state); 
}

```


## Considerations

It would probably be good to get to a place where having explicit lifetimes is not too much of a hassle, 
as this approach does not harness more efficient means of storing only a `&str` compared to a `String`. 

Regardless, a similar approach may be valuable for newcomers to the ratatui community.






