
// TODO simple
// graph-log 
// terminal user interface for graph logging

// [sup  ] ---------------->       [fare-0: ||            ]
//         ---------------->       [fare-1: ||            ]

use std::io::{BufRead, Stdin, stdin};

struct Display {
}


#[derive(Debug)]
enum Log {
  // sender, receiver, args, received
  // sender -[arg1 arg2 arg3]-> receiver [...]
  Send(String, String, Vec<String>, bool),
  
  // name
  // note: it might be already created:
  //  we register it in graph
  // name
  Register(String)

  // TODO

  // name: value
  // Assign(String, String),
  
  // name
  // sender -> _
  //Delete(String),
}


fn tokenize(line: &str) -> Vec<String> {
  // a very simplistic implementation for now
  let mut result = vec![];
  let mut token = String::new();
  for c in line.chars() {
    match c {
      ' ' => {
        if token.len() > 0 {
          result.push(token);
          token = String::new();
        }
      },
      '-' | '>' => {
        if token.len() > 0 {
          result.push(token);
          token = String::new()
        }
        result.push(format!("{}", c));
      },
      _ => {
        token.push(c);
      }
    }
  }
  if token.len() > 0 {
    result.push(token);
  }
  result
}

// TODO Result
fn parse(line: &str) -> Option<Log> {
  // should we use a library?
  // for now just parse simply
  
  // <sender> -
  let tokens = tokenize(line);
  println!("{:?}", tokens);
  if tokens.len() == 0 {
    return None
  }
  let name = tokens[0].clone();
  if tokens.len() > 1 && tokens[1] == "-" {
    // sender -args-> receiver [...]
    // or sender -> receiver [...]
    
    
    if tokens[2] == ">" {
      // sender -> receiver [...]
      let received = tokens.len() == 5 && tokens[4] == "...";
      if !received && tokens.len() != 4 {
        return None
      }
      Some(Log::Send(name, tokens[3].clone(), vec![], received))
    } else {
      // sender -args-> receiver [...]
      let mut args = vec![];
      
      let mut receiver_index = 0usize;
      for i in 2..tokens.len() {
        let token = &tokens[i];
        println!("token {}", &token);
        if token == "-" {
          if i + 2 < tokens.len() && tokens[i + 1] == ">" {
            receiver_index = i + 2;
          } else {
            return None;
          }
          break;
        }
        args.push(tokens[i].clone());
      }
      let received = tokens.len() == receiver_index + 2 && tokens[receiver_index + 1] == "...";
      if !received && tokens.len() != receiver_index + 1 {
        return None
      }
      Some(Log::Send(name, tokens[receiver_index].clone(), args, received))
    }
  } else {
    // name
    if tokens.len() == 1 {
       // name
       Some(Log::Register(name))
    } else {
      None
    }
  }
}

impl Display {
  fn update(&mut self, log: Log) {
     println!("{:?}", log);
     match log {
       Log::Send(sender, receiver, args, received) => {
         println!("[{}] ----[{:?}]------> [{}]", sender, args, receiver);
       },
       Log::Register(name) => {
         println!("[{}]", name);
       }
    }
  }
}

// a | graph_log 
// start a tui here and also output all , but TODO filter [graph-log:] lines
fn graph_log(read: Stdin) {
  let mut display = Display {};
  loop {
    let mut line = String::new();
    match read.read_line(&mut line) {
      Ok(bytes) => {
        if bytes > 0 {
          println!("line {}", line);
          let result = parse(&(line.trim()));
          match result {
             Some(log) => {
               display.update(log);
             },
             None => {} // ignore for now
          }
        } else {
          break;
        }
      },
      Err(_) => {
        break;
      }
    }
  }
} 

fn main() {
  graph_log(stdin());  
}
