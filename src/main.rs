extern crate xmlparser as xml;

use std::env;
use std::fs;
use std::io::Read;
use std::collections::HashMap;
use std::collections::BTreeSet;
use std::io::{BufWriter, Write};

use std::cmp::Ordering;
use std::cmp::Ord;
use std::cmp::PartialEq;

use xml::Token; 

mod circular_buffer;
use circular_buffer::CircularBuffer;

#[derive(Copy, Clone)]
struct BufferedMessage {
    id: u64,
    start: usize,
    end: usize
}

impl Eq for BufferedMessage {

}

impl Ord for BufferedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for BufferedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BufferedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        println!("Usage: parse file.xml output.xml");
        return;
    }

    let text = load_file(&args[1]);
    let output_file_name = &args[2];

    match parse(&text) {
        Err(e) => println!("Parsing error {}", e),
        Ok(msgs) => save_cache_file(output_file_name, &text, msgs).unwrap()
    }
}

fn parse(text: &str) -> Result<BTreeSet<BufferedMessage>, xml::Error> {
    let mut count = 0;

    let mut usr_buffers: HashMap<i32, CircularBuffer<BufferedMessage>> = HashMap::with_capacity(20);

    let mut message_count = 0;
    let mut dest: Vec<i32> = Vec::with_capacity(50);

    let mut msg = BufferedMessage { id: 0, start: 0, end: 0};
    for token_option in xml::Tokenizer::from(text) {
        //println!("{:?}", token?);
        let token = token_option.unwrap();
        match token {
            Token::ElementStart{prefix: _, local, span} => {
                match local.as_str() {
                    "msg" => { 
                        message_count += 1;
                        msg.id =  message_count;
                        msg.start = span.start();
                    },
                    "to" => {},
                    _ => ()
                }
            },
            Token::Attribute{prefix:_, local, value, span:_} => {
                if let "uid" = local.as_str() {
                    if let Ok(uid) = value.as_str().parse() {
                        dest.push(uid);
                    }
                }
            },
            Token::ElementEnd {end, span} => match end {
                xml::ElementEnd::Close (_prefix, local) => {
                    if local.as_str() == "msg" {
                        msg.end = span.end();
                        // insert the message in all destination buffers
                        for d in dest.iter() {
                            match usr_buffers.get_mut(d) {
                                Some(buf) => buf.add(msg),
                                None => {
                                    let mut buf = CircularBuffer::new(10);
                                    buf.add(msg);
                                    usr_buffers.insert(*d, buf);
                                }
                            }
                        }
                        // clean up the destinations
                        dest.clear();
                    } 
                },
                _ => {}
            },
            _ => ()
        }
        count += 1;
    }

    let mut msgs:BTreeSet<BufferedMessage> = BTreeSet::new();
    for (_, buf) in usr_buffers.iter() {
        for buf_msg in buf.iter() {
            msgs.insert(*buf_msg);
        }
    }

    println!("Tokens {}, total messages: {}, cached messages: {}", count, message_count, msgs.len());
    Ok(msgs)
}

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}

// writing chatcache.xml
fn save_cache_file(output_file_name: &str, text: &str, msgs: BTreeSet<BufferedMessage>) -> Result<(), std::io::Error> {
    let mut f = BufWriter::with_capacity(8 * 1024, fs::File::create(output_file_name)?);
    let eol: [u8; 2] = [13, 10];
    f.write(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    f.write(&eol)?;
    f.write(b"<root>")?;
    f.write(&eol)?;
    for msg in msgs.iter() {
        let s = &text[msg.start .. msg.end];
        f.write(&*s.as_bytes())?;
        f.write(&eol)?;
    }
    f.write(b"</root>")?;
    f.write(&eol)?;
    f.flush()?;
    Ok(())
}