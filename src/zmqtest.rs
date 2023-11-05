use zmq::{Context, Socket, DONTWAIT, POLLIN};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
use ctrlc;



fn subscriber() -> Result<(),  Box<dyn Error> > {
    let running = Arc::new(AtomicBool::new(true));

    // 处理 Ctrl+C 信号
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let context = Context::new();
    let subscriber = context.socket(zmq::SUB)?;
    subscriber.connect("tcp://localhost:5555")?;
    subscriber.set_subscribe(b"")?;

    let (tx, rx) = mpsc::channel();

    // 创建一个线程来接收消息
    let receiver = std::thread::spawn(move ||   {
        let mut items = [subscriber.as_poll_item(POLLIN)];
        loop {
            if !running.load(Ordering::SeqCst) {
                break; // 收到信号后跳出循环
            }

            match zmq::poll(&mut items, Duration::from_millis(100).as_secs() as i64) {
                Ok(_) => {
                    if items[0].is_readable() {
                        // let message = subscriber.recv_msg(DONTWAIT)?;
                        let message = match subscriber.recv_msg(DONTWAIT) {
                            Ok(msg) => msg,
                            Err(err) => return Err(Box::new(err) ),
                        };

                        let message_str = message.as_str().unwrap();
                        tx.send(message_str.to_owned()).unwrap();
                    }
                }
                Err(err) => {
                    return Err(err.into());

                }
            }
        }
        Ok(())
    });

    // 主线程，等待 Ctrl+C 信号
    // while running.load(Ordering::SeqCst) {
    //     std::thread::sleep(std::time::Duration::from_millis(100));
    // }

    // 停止接收消息的线程
    receiver.join().unwrap()?;

    Ok(())
}

pub fn test_main() {
    if let Err(err) = subscriber() {
        eprintln!("Error: {}", err);
    }
}