use serde::Deserialize;

pub trait Servable {
    fn toString(&self) -> String;
}

pub struct Incrementer {
    pub val: i64,
}

pub struct Listener {
    pub subject: String,
    pub reciever: tokio::sync::mpsc::UnboundedSender<std::result::Result<warp::ws::Message, warp::Error>>
}

impl Listener {
    pub fn send(&self, subject: String, value: i64) -> () {
        self.reciever.send(Ok(warp::ws::Message::text(format!("subject:{}|value:{}", subject, value))));
    }
}

impl Incrementer {
    pub fn increment(&mut self) -> Option<&Incrementer> {
        self.val+=1;
        println!("new value {}", self.val);
        return Some(self)
    }
}

impl Servable for Incrementer {
    fn toString(&self) -> String {
        self.val.to_string()
    }
}

