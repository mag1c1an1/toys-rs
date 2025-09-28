use anyhow::Result;
use lag::{Calculator, Message};
use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
};

struct Server {
    addr: String,
    finish: bool,
    cal: Calculator,
}

impl Server {
    fn new(addr: &str) -> Result<Self> {
        Ok(Self {
            addr: addr.to_string(),
            finish: false,
            cal: Calculator::new(),
        })
    }
    fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        for stream in listener.incoming() {
            let stream = stream?;
            self.handle_connection(stream)?;
            if self.finish {
                self.cal.print();
                break;
            }
        }
        Ok(())
    }
    fn handle_connection(&mut self, mut stream: TcpStream) -> Result<()> {
        let buf_reader = BufReader::new(&mut stream);
        let content: Vec<_> = buf_reader
            .lines()
            .map(|res| res.unwrap())
            .take_while(|line| !line.is_empty())
            .flat_map(|s| s.into_bytes())
            .collect();
        let s = std::str::from_utf8(&content)?;
        let msg: Message = ron::from_str(s)?;
        match msg {
            Message::Micro(val) => self.cal.add_msg(val)?,
            Message::Finish => self.finish = true,
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut server = Server::new("127.0.0.1:33443")?;
    server.start()?;
    println!("server shutdown");
    Ok(())
}
