use chrono::Local;
use ctrlc;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use zmq::{Context, Socket, SocketType, DONTWAIT, POLLIN};
use crate::message::{decodeMessage,encodeMessage,MessagePosition};

fn subscriber(endpoint: &str, msg_back: fn(&[u8])) -> Result<(), Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));

    // 处理 Ctrl+C 信号
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let context = Context::new();
    let subscriber = context.socket(zmq::SUB)?;
    subscriber.set_subscribe(b"")?;
    subscriber.connect(endpoint)?;
    println!("subscriber()...2");
    // let (tx, rx) = mpsc::channel();

    // 创建一个线程来接收消息
    let receiver = std::thread::spawn(move || {
        let mut items = [subscriber.as_poll_item(POLLIN)];
        loop {
            match zmq::poll(&mut items, Duration::from_millis(50000).as_secs() as i64) {
                Ok(_) => {
                    if items[0].is_readable() {
                        // println!("some data readable()...3");
                        let message = match subscriber.recv_bytes(DONTWAIT) {
                            Ok(msg) => msg,
                            Err(err) => return Err(Box::new(err)),
                        };

                        // let message_str = message.as.unwrap();
                        // tx.send(message).unwrap();
                        msg_back(&message);
                    }
                }
                Err(err) => {
                    println!("zmq::pool:{}", err.message());
                    return Err(err.into());
                }
            }
        }
        Ok(())
    });

    // 主线程，等待 Ctrl+C 信号
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 停止接收消息的线程
    receiver.join().unwrap()?;

    Ok(())
}

fn parseMessage(message:&[u8]){
    // let now = Local::now();
    // let timestr = now.format("%Y-%m-%d %H:%M:%S").to_string();
    // println!(">> {} len:{} , {:?}", timestr,message.len(), message);
    if let Some(pos) = decodeMessage(message){
        println!("message :{:#?}",pos);
    }else{
        println!("message invalid. {:?}",message);
    }
}

// #[test]
pub fn test_main() {
    if let Err(err) = subscriber("tcp://127.0.0.1:15556",parseMessage) {
        eprintln!("Error: {}", err);
    }
    println!("test_main end..");
}
