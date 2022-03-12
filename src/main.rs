mod types;
mod container;
mod controllers;

use std::collections::HashMap;
use warp::Filter;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() {

    //TODO: better error handling, proper status codes/request methods
    
    let storage: IncStoreRef = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let subscribers: SubscriberData = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let route = routes(storage, subscribers);

    println!("Up and running");

    warp::serve(route)
        .run(([0,0,0,0], 5001))
        .await;

}

fn add_store(isr: IncStoreRef) -> impl Filter<Extract = (IncStoreRef,), Error = std::convert::Infallible> + Clone
{
    return warp::any().map(move || isr.clone());
}

fn add_subscribers(subs: SubscriberData)  -> impl Filter<Extract = (SubscriberData,), Error = std::convert::Infallible> + Clone 
{
    return warp::any().map(move || subs.clone());   
}


fn routes(store: IncStoreRef, subscribers: SubscriberData) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    return route_add_store(store.clone())
        .or(route_list_stores(store.clone()))
        .or(route_inspect_store(store.clone()))
        .or(route_increment_store(store.clone(), subscribers.clone()))
        .or(route_subscribe_to_updates(store.clone(), subscribers.clone()))
    ;
}

fn route_add_store(store: IncStoreRef) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    return warp::path!("store" / "create" / "increment" / String )
        .and(warp::get()) //dont mind me, just so i can check it from browser quickly
        .and(add_store(store))
        .and_then(controllers::stores::create_increment)
    ;
}

fn route_inspect_store(store: IncStoreRef) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    return warp::path!("store" / "inspect" / String)
        .and(warp::get())
        .and(add_store(store))
        .and_then(controllers::stores::inspect_store)
    ;
}

fn route_list_stores(store: IncStoreRef) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    return warp::path!("store" / "inspect")
        .and(warp::get())
        .and(add_store(store))
        .and_then(controllers::stores::list_stores)
    ;
}

fn route_increment_store(store: IncStoreRef, subscribers: SubscriberData) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{ 
    return warp::path!("store" / "update" / String)
        .and(warp::get()) //dont mind me, just so i can check it from browser quickly
        .and(add_store(store))
        .and(add_subscribers(subscribers))
        .and_then(controllers::stores::increment_store)
    ;
}

fn route_subscribe_to_updates(store: IncStoreRef, subscribers: SubscriberData) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    return warp::path!("store" / "listen" / String)
        .and(warp::ws())
        .and(add_store(store.clone()))
        .and(add_subscribers(subscribers.clone()))
        .and_then(controllers::stores::connect_to_socket)
    ;
}


type IncStoreRef = Arc<tokio::sync::Mutex<HashMap<String, container::StateContainer<types::Incrementer>>>>;
type SubscriberData = Arc<tokio::sync::Mutex<HashMap<String, types::Listener>>>; //TODO: move