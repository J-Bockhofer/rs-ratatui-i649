use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use regex::Regex;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};


// Code for stateful list from 
// 
#[derive(Default)]
struct StatefulList<T> {
  state: ListState,
  items: Vec<T>,
}

impl<T> StatefulList<T> {
  fn with_items(items: Vec<T>) -> StatefulList<T> {
      StatefulList {
          state: ListState::default(),
          items,
      }
  }

  fn next(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i >= self.items.len() - 1 {
                  0
              } else {
                  i + 1
              }
          }
          None => 0,
      };
      //println!("next Item: {i}");
      self.state.select(Some(i));
  }

  fn previous(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i == 0 {
                  self.items.len() - 1
              } else {
                  i - 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

  fn unselect(&mut self) {
      self.state.select(None);
  }

  fn trim_to_length(&mut self, max_length: usize) {
    while self.items.len() > max_length {
        self.items.remove(0);
    }
  }
}





#[derive(Default)]
pub struct Home <'a>{
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  iostreamed: StatefulList<String>,

  available_actions: StatefulList<&'a str>, // &str is optional here but since the example in list.rs uses it, I did want to include it to show that complications with explicit lfietimes can arise quickly
}

impl<'a> Home<'a>{
  pub fn new() -> Self {
    Self::default().set_default_actions()
  }

  pub fn set_default_actions(mut self) -> Self {
    self.available_actions = StatefulList::with_items(vec![
      "Action 1",
      "Action 2",
      "Action 3",
    ]);
    self
  }
}

impl Component for Home<'_> {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {
      },
      Action::IONotify(x) => {
        self.iostreamed.items.push(x.clone());
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {

    let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
    .split(f.size());

    // If REGEX are defined here the problem becomes much less noticable! So moving them further out would be even better.

    let iolines: Vec<ListItem> = self
    .iostreamed
    .items
    .iter()
    .map(|i| {
        // split regex logic here
        let collected: Vec<&str> = i.split("++++").collect();
        let mut lines: Line = Line::default();

        for line in collected {
          let mut splitword: &str = "(/%&$§"; // default splitword that shouldnt do anything

          // REGEX were defined here in my scratch solution
          let ip_re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(); // IPv4 regex
          let ban_re = Regex::new(r"Ban").unwrap();
          let found_re = Regex::new(r"Found").unwrap();

          let results: Vec<&str> = ip_re
            .captures_iter(&line)
            .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
            .collect();
          let cip: &str;
          if !results.is_empty() {
            // assume only left and right side - not multiple ips in one line
            // assume splitword is on left of ip --- layout of fail2ban sshd
            cip = results[0];

            if ban_re.is_match(&line) {splitword = "Ban";}
            else if found_re.is_match(&line)  {splitword = "Found";}
            let fparts: Vec<&str> = line.split(cip).collect();
            let sparts: Vec<&str> = fparts[0].split(splitword).collect();

            let startspan = Span::styled(sparts[0], Style::default().fg(Color::White));
            lines.spans.push(startspan);

            if sparts.len() > 1 {
              // Found or Ban
              let splitspan = Span::styled(format!("{} ",splitword), Style::default().fg(Color::LightCyan));
              lines.spans.push(splitspan);
            }
            if fparts.len() > 1 {
              let ipspan = Span::styled(cip, Style::default().fg(Color::LightRed));
              lines.spans.push(ipspan);
              let endspan = Span::styled(fparts[1], Style::default().fg(Color::White));
              lines.spans.push(endspan);
            }
          }
        }
        ListItem::new(lines).style(Style::default().fg(Color::White))
    })
    .collect();

    let iolist_title = Line::from(" I/O Stream ");
    // Create a List from all list items and highlight the currently selected one
    let iolist = List::new(iolines)
        .block(Block::default()
          .borders(Borders::ALL)
          .border_style( Style::new().white())
          .title(iolist_title)
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");


      let av_actions: Vec<ListItem> = self
      .available_actions
      .items
      .iter()
      .map(|i| {  
          ListItem::new(Line::from(*i)).style(Style::default().fg(Color::White))
      })
      .collect();
  
      // Create a List from all list items and highlight the currently selected one
      let actionlist = List::new(av_actions)
          .block(Block::default()
          .borders(Borders::ALL)
          .border_style( Style::new().white())
          .title("Actions"))
          .highlight_style(
              Style::default()
                  .fg(Color::Black)
                  .bg(Color::LightGreen)
                  .add_modifier(Modifier::BOLD),
          )
          .highlight_symbol(">> ");

      f.render_stateful_widget(actionlist, layout[0], &mut self.available_actions.state);

    f.render_stateful_widget(iolist, layout[1], &mut self.iostreamed.state);

    Ok(())
  }
}

