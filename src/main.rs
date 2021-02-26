
// TODO simple
// graph-log 
// terminal user interface for graph logging

// [sup  ] ---------------->       [fare-0: ||            ]
//         ---------------->       [fare-1: ||            ]

use std::io::{BufRead, Stdin, Stdout, stdin, stdout};
use std::{thread, time};
use tui::{
  backend::{
//    RustboxBackend
      TermionBackend
  },
  layout::Layout,
  widgets::{LineGauge, Block, Borders, Clear, canvas::{Canvas, Rectangle, Line, Label}},
  symbols,
  style::{Color},
  Terminal,
  Frame
};
use termion::{
  input::{TermRead},
  raw::{IntoRawMode, RawTerminal}
};
use std::fs;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

struct Display {
  // a terminal: the `tui` and currently `termion`, eventually `rustbox` -based type for work with terminal
  terminal: Terminal<TermionBackend<RawTerminal<Stdout>>>, // <RustboxBackend>
  
  // graphs that we display
  graphs: HashMap<String, Graph>,

  // connections between graphs with some state
  connections: HashMap<String, Connection>,

  // history of logs
  logs: Vec<Log>,

  // the active log index
  logIndex: usize,

  // default node neight
  node_height: f64,

  // default node width
  node_width: f64,

  // static str
  text: &'static str
}

#[derive(Clone, Debug)]
struct Graph {
  // the name of the graph
  name: String,

  // the nodes inside the graph
  nodes: HashMap<String, Node>,

  // the x coord of the first nodes of the graph
  x: f64
}


#[derive(Clone, Debug)]
struct Connection {
  // where the connection starts
  from: (String, String),

  // where the connection ends
  to: (String, String),

  // args for the sended message
  args: Vec<String>,

  // joinned text of the args
  text: &'static str,

  // received or still sending
  status: MessageStatus,

  // a left x coord
  x_start: f64,

  // a right x coord
  x_end: f64,

  // a first y coord
  y_start: f64,

  // an last y coord
  y_end: f64,

  // a left args x coord
  x_args_start: f64,

  // a right args x coord
  x_args_end: f64,

  // a first args y coord
  y_args_start: f64,

  // a last args y coord
  y_args_end: f64
}

#[derive(Clone, Debug)]
enum MessageStatus {
  Received,
  Sending,
  Error,
  Warning
}

#[derive(Clone, Debug)]
struct Node {
  // the name of the node
  name: String,

  // the value in the node
  value: String,

  // a lower left x coord
  x: f64,

  // a lower left y coord
  y: f64
}

#[derive(Debug)]
enum Log {
  // sender (sender graph, sender node), receiver (receiver graph, receiver node), args, received
  // sender -[arg1 arg2 arg3]-> receiver [...]
  Send((String, String), (String, String), Vec<String>, bool),
  
  // name (graph, node)
  // note: it might be already created:
  //  we register it in graph
  // name
  Register((String, String))

  // TODO

  // name: value
  // Assign(String, String),
  
  // name
  // sender -> _
  //Delete(String),
}

const WAIT_TIME_IN_MS: u64 = 300;

fn tokenize(line: &str) -> Vec<String> {
  // a very simplistic implementation for now
  // <space>- is special
  // dont delimit on space in ""
  let mut result = vec![];
  let mut token = String::new();
  let mut in_string = false;
  for (i, c) in line.chars().enumerate() {
    if !in_string {
      match c {
        ' ' => {
          if token.len() > 0 {
            result.push(token);
            token = String::new();
          }
        },
        '-' => {
          if i > 0 && line.chars().nth(i - 1).unwrap() == ' ' || i < line.len() - 1 && line.chars().nth(i + 1).unwrap() == '>'  {
            if token.len() > 0 {
              result.push(token);
              token = String::new();
            }

            result.push("-".to_string());
          } else {
            token.push(c);
          }
        },
        '>' => {
          if token.len() > 0 {
            result.push(token);
            token = String::new()
          }
          result.push(format!("{}", c));
        },
        _ => {
          if c == '"' {
            in_string = true;
          }
          token.push(c);
        }
      }
    } else {
      if c == '"' {
        in_string = false;
      }
      token.push(c);
    }
  }
  if token.len() > 0 {
    result.push(token);
  }
  result
}

fn cleanup(token: &str) -> String {
  // TODO non-ascii
  let mut result: Vec<char> = token.trim().to_string().chars().collect();
  if result[0] == '"' && result[result.len() - 1] == '"' ||
     result[0] == '\'' && result[result.len() - 1] == '\'' {
    result = result[1 .. result.len() - 1].to_vec()
  }
  result.iter().collect()
}

// graph:node or node 
//   -> (graph, node) or ("top-level", node)
fn to_tuple(token: &str) -> (String, String) {
  let tokens = token.split(':').map(|t| t.to_string()).collect::<Vec<String>>();
  if tokens.len() == 2 {
    (tokens[0].clone(), tokens[1].clone())
  } else if tokens.len() == 1 {
    ("top-level".to_string(), tokens[0].clone())
  } else {
    (tokens[0].clone(), tokens[1 .. tokens.len() - 1].join(":"))
  }
}

// TODO Result
fn parse(line: &str) -> Option<Log> {
  // should we use a library?
  // for now just parse simply
  
  // <sender> -
  let tokens = tokenize(line);
  // println!("{:?}", tokens);
  if tokens.len() == 0 {
    return None
  }

  let name = to_tuple(&cleanup(&tokens[0]));
  if tokens.len() > 1 && tokens[1] == "-" {
    // sender -args-> receiver [...]
    // or sender -> receiver [...]
    
    
    if tokens[2] == ">" {
      // sender -> receiver [...]
      let received = tokens.len() == 5 && tokens[4] == "...";
      if !received && tokens.len() != 4 {
        return None
      }
      Some(Log::Send(name, to_tuple(&cleanup(&tokens[3])), vec![], received))
    } else {
      // sender -args-> receiver [...]
      let mut args = vec![];
      
      let mut receiver_index = 0usize;
      for i in 2..tokens.len() {
        let token = &tokens[i];
        // println!("token {}", &token);
        if token == "-" {
          if i + 2 < tokens.len() && tokens[i + 1] == ">" {
            receiver_index = i + 2;
          } else {
            return None;
          }
          break;
        }
        args.push(cleanup(&tokens[i]));
      }
      let received = tokens.len() == receiver_index + 2 && tokens[receiver_index + 1] == "...";
      if !received && tokens.len() != receiver_index + 1 {
        return None
      }
      Some(Log::Send(name, to_tuple(&cleanup(&tokens[receiver_index])), args, received))
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

  fn start<'a>(&'a mut self) {
    let layout = Layout::default();
    let graphs = &self.graphs;
    let connections = &self.connections;
    let node_height = self.node_height;
    let node_width = self.node_width;
    // let text_clone = self.connections["top-level:sup fare:0"].text.clone();
    // let text = &text_clone;
    let result = self.terminal.draw(|frame| {
      let size = frame.size();
      let default_color = Color::White;
      let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("example"))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            //
            // for graph in graphs:
            //   for node in graph.nodes:
            //     render a rectangle with the node coords and sizes, a default color for now
            //     print the text and value in node coords + 2.0 
            // for connection in connections:
            //   render a line from the start coords to end coords and color depending on received
            //
            // let graphs = self.graphs; //.clone();
            for (name, graph) in graphs.iter() { //lock().unwrap().iter() {
              for (node_name, node) in graph.nodes.iter() {
                let rectangle = Rectangle { x : node.x, y: node.y,  height: node_height, width: node_width, color: default_color };
                ctx.draw(&rectangle);
                // let rectangle2 = Rectangle { x : 0.0, y : 0.0, height: 5.0, width: 5.0, color: Color::Black };
                // ctx.draw(&rectangle2);
                let text = format!("{} | {}", &node.name, &node.value);
                // names.push(text.clone());
                // TODO text
                ctx.print(node.x + 2.0, node.y + 2.0, Box::leak(node.name.clone().into_boxed_str()), default_color);
              }
            }
            for (names, connection) in connections.iter() {
              let color = default_color; // { default_color } else { sending_color };
              let mut lines: Vec<Line> = vec![];
              // let mut text = "".to_string();
              let text = connection.text;
              if connection.args.len() == 0 {
                let line_no_args = Line { x1: connection.x_start, y1: connection.y_start, x2: connection.x_end, y2: connection.y_end, color: color };
                lines.push(line_no_args);
              } else {
                let line_before_args = Line { x1: connection.x_start, y1: connection.y_start, x2: connection.x_args_start, y2: connection.y_args_start, color: color };
                let line_after_args = Line { x1: connection.x_args_end, y1: connection.y_args_end, x2: connection.x_end, y2: connection.y_end, color: color };
                lines.push(line_before_args);
                lines.push(line_after_args);
                // texts.push(connection.args.join(" "));
              }
              for line in lines {
                ctx.draw(&line);
              }
              if text.len() > 0 {

                ctx.print(connection.x_args_start + 5.0, connection.y_args_start + (connection.y_args_end - connection.y_args_start) * 0.5, text, color);
              }
            }

            // ctx.draw(&Rectangle {
            //   x: 0.0, y: 0.0, height: node_height, width: node_width, color: Color::Black
            // });
            // ctx.draw(&Line {
            //   x1: node_width,
            //   y1: node_height / 2.0,
            //   x2: 50.0,
            //   y2: node_height / 2.0,
            //   color: Color::Black
            // });
            // ctx.print(
            //   1.0,
            //   1.0,
            //   "node",
            //   Color::Black
            // );

            // for i in 0 .. 10 {
            //   let x = 50.0;
            //   let y = 100.0 - (i as f64 * node_height * 3.0) - 10.0;
            //   ctx.draw(&Rectangle {
            //     x: x,
            //     y: y,
            //     height: node_height,
            //     width: node_width,
            //     color: Color::Black
            //   });
            //   // let name = format!("node-{}", i);
            //   // names.push(name);
            //   ctx.print(
            //     x + 2.0,
            //     y + 2.0,
            //     "node i",
            //     Color::Black
            //   );
            // }
         });
        // .line_set(symbols::line::THICK);
      
      
      frame.render_widget(Clear, size);
      frame.render_widget(canvas, size);
    });
//    println!("{:?}", result);
    // TODO result
  }

  fn update(&mut self, log: Log) {
    // println!("{:?}", log);
    match log {
      Log::Send(sender, receiver, args, received) => {
        self.check(sender.clone());
        self.check(receiver.clone());
        let name = format!("{}:{} {}:{}", sender.0, sender.1, receiver.0, receiver.1);
        // println!("{}", name);
        let status = if received { MessageStatus::Received } else { MessageStatus::Sending };
        if !self.connections.contains_key(&name) {
          // [ a ] \
          //        \->  [ b ]
          // sender_node.x + node_width, sender_node.y + node_height / 2.0, receiver_node.x, receiver_node.y + node_height / 2.0
          // println!("{:?}", self.graphs);
          let sender_node = &self.graphs[&sender.0].nodes[&sender.1];
          let receiver_node = &self.graphs[&receiver.0].nodes[&receiver.1];
          // two connections or just no:
          // one connection but write eventually two lines: x0 y0: x0 + (x1-x0) * 0.4, y0 + (y1-y0) * 0.4 and x0 + (x1-x0) * 0.6 y0 + (y1 - y0) * 0.6
          let text = args.join(" ");
          let mut connection = Connection { 
            from: sender.clone(),
            to: receiver.clone(),
            args: args.clone(),
            // TODO: change eventually
            text: Box::leak(text.into_boxed_str()),
            status: status,
            x_start: sender_node.x + self.node_width,
            y_start: sender_node.y + self.node_height / 2.0,
            x_end: receiver_node.x,
            y_end: receiver_node.y + self.node_height / 2.0,
            x_args_start: 0.0,
            y_args_start: 0.0,
            x_args_end: 0.0,
            y_args_end: 0.0
          };
          connection.x_args_start = connection.x_start + (connection.x_end - connection.x_start) * 0.4;
          connection.y_args_start = connection.y_start + (connection.y_end - connection.y_start) * 0.4;
          connection.x_args_end = connection.x_start + (connection.x_end - connection.x_start) * 0.6;
          connection.y_args_end = connection.y_start + (connection.y_end - connection.y_start) * 0.6;

          // println!("{:?}", connection);
          // println!("{:?}", connection);
          self.connections.entry(name).or_insert(connection);
        }
      },
      Log::Register(name) => {
        self.register(name);
        // println!("{:?}", self.graphs);
      }
    }
    self.start();
  }

  fn check(&mut self, pair: (String, String)) {
    let (graph, node) = pair.clone();
    if !self.graphs.contains_key(&graph) || !self.graphs[&graph].nodes.contains_key(&node) {
      self.register(pair);
    }
  }

  fn register(&mut self, pair: (String, String)) {
    // register graph if not existing
    // register node with x: the graphs' x and y: the number of nodes in graph  * node_height * 3.0
    let (graph_name, node_name) = pair;
    if !self.graphs.contains_key(&graph_name) {
      self.register_graph(graph_name.clone());
    }
    let x = self.graphs[&graph_name].x;
    let y = self.graphs[&graph_name].nodes.len() as f64 * self.node_height * 3.0;
    let node_value = Node { name: node_name.clone(), value: "".to_string(), x: x, y: y };
    // println!("node {:?}", node_value);
    let mut graph_value = self.graphs[&graph_name].clone();
    graph_value.nodes.insert(node_name, node_value);
    // self.graphs.entry(graph_name).or_insert(graph_value.clone());
    self.graphs.insert(graph_name, graph_value.clone());
    // println!("graphs {:?}", graph_value);
  }

  fn register_graph(&mut self, name: String) {
    // register graph with x: the next 50% for now
    // TODO better algorithm for more graphs
    // println!("graph {}", name);
    let x = if self.graphs.len() == 0 { 0.0 } else { 50.0 };
    let graph = Graph { name: name.clone(), nodes: HashMap::new(), x: x };
    self.graphs.entry(name).or_insert(graph);
  }
}

// TODO
// the pipe approach: a little bit too hard for stdin
// a | graph_log
// for now:
// graph_log <filename>
// start a tui here and also output all , but TODO filter [graph-log:] lines

//fn graph_log(read: Stdin) {
fn graph_log(file: &str) {
  let text = fs::read_to_string(file).unwrap();
  //let backend = RustboxBackend::new().unwrap();
  let stdout_stream = stdout().into_raw_mode().unwrap();
  let backend = TermionBackend::new(stdout_stream);
  let mut terminal = Terminal::new(backend).unwrap();
  terminal.autoresize().unwrap();
  terminal.clear().unwrap();
  let mut display = Display { terminal: terminal, graphs: HashMap::new(), connections: HashMap::new(), logIndex: 0usize, logs: vec![], node_width: 5.0, node_height: 5.0, text: "" };
  display.start();
  
  // TODO
  // stdin/pipe
  //loop {
    // let mut line = String::new();
    // match read.read_line(&mut line) {
  for line in text.lines() {
      // Ok(bytes) => {
        // if bytes > 0 {
      
        if line.len() > 0 {
          // println!("line {}", line);
          // 11
          if line.len() < 11 {
            continue;
          }
          let raw_line = &line["[graph-log]".len() ..];
          let result = parse(&(raw_line.trim()));
          // println!("{:?}", result);
          match result {
             Some(log) => {
               display.update(log);
               thread::sleep(time::Duration::from_millis(WAIT_TIME_IN_MS));
             },
             None => {} // ignore for now
          }
        // } else {
          // break;
        }
      // },
      // Err(_) => {
        // break;
      // }
    // }
  }
  // loop {}
  // thread::sleep(time::Duration::from_millis(5_000));
} 

fn main() {
  graph_log("/home/alexander/serviceupdater/sup/log");  
}
