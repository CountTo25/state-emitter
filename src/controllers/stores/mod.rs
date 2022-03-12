use crate::types;
use crate::container;
use std::sync::Arc;
use std::collections::HashMap;
use std::convert::Infallible;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use warp::{ws::Message, Error as WarpError};
use std::result::Result;
use futures::{FutureExt, StreamExt};



pub async fn create_increment(
    key: String,
    store: Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>,
) -> Result<impl warp::Reply, Infallible>
{
    let mut open_store =  store.lock().await;//.insert(key, value);
    let incrementer = container::create_state_container(
        types::Incrementer {val: 0}
    );
    let val = open_store.insert(key.to_string(), incrementer).unwrap();
    let mut message: HashMap<String, String> = HashMap::new();
    message.insert("success".to_string(), "OK".to_string()); //dont mind me
    return Ok(warp::reply::json(&message));
}


pub async fn inspect_store(
    name: String,
    store: Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>,
) -> Result<impl warp::Reply, Infallible> 
{
    let open_store = &*store.lock().await;
    let key_value = open_store.get(&name).unwrap();
    return Ok(warp::reply::json(&key_value));
}

pub async fn list_stores(
    store: Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>
) -> Result<impl warp::Reply, Infallible> 
{
    let open_store = &*store.lock().await;
    return Ok(warp::reply::json(&open_store));
}

pub async fn increment_store(
    name: String,
    store: Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>,
    subscribers: Arc<tokio::sync::Mutex<HashMap<String, types::Listener>>>,
) -> Result<impl warp::Reply, Infallible>  
{
    let open_store = &*store.lock().await;
    match open_store.get(&name).unwrap().increment() {
        Ok(val) => {
            let ready = open_store.get(&name).unwrap();
            send_update(subscribers, name, val).await;
            return Ok(warp::reply::json(&ready));
        },
        Err(e) => {
            let mut message: HashMap<String, String> = HashMap::new();
            message.insert(String::from("error"), String::from(e));
            return Ok(warp::reply::json(&message));
        }
    }
    
}

pub async fn connect_to_socket(
    subject: String,
    socket: warp::ws::Ws,
    store: Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>,
    subs: Arc<tokio::sync::Mutex<HashMap<String, types::Listener>>>,
) -> Result<impl warp::Reply, Infallible>  
{
    Ok(socket.on_upgrade(move |ws| establish_connection(subject, ws, store, subs)))
} 

async fn establish_connection(
    subject: String,
    socket: warp::ws::WebSocket,
    store: Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>,
    subs: Arc<tokio::sync::Mutex<HashMap<String, types::Listener>>>,
) 
{
        let (wss, wsr) = socket.split();
        let (cs, cr): (
            UnboundedSender<Result<Message, WarpError>>,
            UnboundedReceiver<Result<Message, WarpError>>
        ) //fucking hell, TODO: make this compact and better, import via {}, move to types
             = tokio::sync::mpsc::unbounded_channel();
        let cr = tokio_stream::wrappers::UnboundedReceiverStream::new(cr); //wrap

        tokio::task::spawn(cr.forward(wss).map(|res| {
            if let Err(e) = res {
                eprintln!("error sending websocket msg: {}", e);
            }
        }));

        let mut existing_subs = subs.lock().await;
        let id: String = uuid::Uuid::new_v4().to_string();

        existing_subs.insert(id, types::Listener {
            subject,
            reciever: cs,
        });

        println!("New connection OK");

        return ();
}

async fn send_update( 
    subs: Arc<tokio::sync::Mutex<HashMap<String, types::Listener>>>,
    subject: String,
    value: i64, //so much for flexibility 2;
) {
    let subs = subs.lock().await;
    subs.iter()
        .filter(|(_key, client)| client.subject == subject)
        .for_each(|(_key, client)| {
            client.send(subject.to_string(), value);
            return ();
        });

    return (); 
}