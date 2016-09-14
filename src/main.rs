extern crate rustc_serialize;
extern crate telegram_bot;
extern crate getopts;

use rustc_serialize::json;
use telegram_bot as tgram;
use getopts::Options;

use std::io::prelude::*;
use std::fs::File;
use std::env;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct BotConfig {
    pub token: String,
    pub commands: Vec<Command>,
    pub keywords: Vec<Keyword>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Command {
    pub name: String,
    pub reply: String,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Keyword {
    pub word: String,
    pub reply: String,
}

impl BotConfig {
    fn from_default_file(path: &str) -> BotConfig {
        let mut f = File::open(path).unwrap();
        let mut encoded = String::new();
        f.read_to_string(&mut encoded).unwrap();
        let decoded: BotConfig = json::decode(&encoded).unwrap();
        decoded
    }
}

pub struct Dumbot {
    api: tgram::Api,
    config: BotConfig,
}

impl Dumbot {
    fn new(config: BotConfig) -> Dumbot {
        let  api = tgram::Api::from_token(&config.token).unwrap();
        println!("Name: {:?}", api.get_me().unwrap().first_name);
        Dumbot {
            api: api,
            config: config,
        }
    }

    fn listen(&mut self) {
        let mut listener = self.api.listener(tgram::ListeningMethod::LongPoll(None));
        let res = listener.listen(move |u| {
            if let Some(m) = u.message {
                let name = m.from.first_name;

                match m.msg {
                    tgram::MessageType::Text(t) => {
                        println!("<{}> {}", name, t);
                        
                        for keyword in self.config.keywords.iter() {
                            let kw = keyword.word.to_lowercase();
                            if t.to_lowercase().contains(&kw) {
                                try!(self.api.send_message(
                                        m.chat.id(),
                                        keyword.reply.clone(),
                                        None, None, None, None));
                            }
                        }

                    },
                    _ => {}
                }
            }
            Ok(tgram::ListeningAction::Continue)
        });

        if let Err(e) = res {
            println!("Am error occured: {}", e);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("c", "config", "config file", "config.json");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    let config = matches.opt_str("c").unwrap_or(String::from("config.json"));

    let config = BotConfig::from_default_file(&config);
    let mut bot = Dumbot::new(config);
    bot.listen();
    println!("OK!");

    println!("Token: {}", bot.config.token);
    println!("{:?}", bot.config);
}
