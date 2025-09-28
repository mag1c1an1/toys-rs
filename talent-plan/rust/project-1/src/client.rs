use std::io::{BufReader, BufWriter};
use std::net::{TcpStream, ToSocketAddrs};
use serde_json::de::IoRead;
use serde_json::Deserializer;
use crate::Result;


pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}


impl KvsClient {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        todo!()
    }
    pub fn get(&mut self, key: String) -> Result<Option<String>> { todo!() }
    pub fn set(&mut self, key: String) -> Result<()> { todo!() }
    pub fn remove(&mut self, key: String) -> Result<()> { todo!() }
}