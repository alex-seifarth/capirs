use capirs::*;
use std::sync::Arc;

#[tokio::main]
pub async fn main() {
    let connection = capirs::Connection::create("connection-app").unwrap();
    connection.start(true).await;
    let (quit_s, mut quit_r) = tokio::sync::broadcast::channel::<bool>(4);

    let conn = connection.clone();
    let tsk = tokio::spawn(async move {
        let mut channel = tokio::sync::mpsc::channel(1024);
        let svc = capirs::ServiceInstanceID{
            service: 0x1111, instance: 0x2222, major_version: 0x01, minor_version: someip::DEFAULT_MINOR
        };
        let sender = channel.0.clone();
        let result = conn.register_proxy(svc, sender).await;
        assert!(result.is_ok());
        let proxy_id = match result {
            Ok(id) => id,
            Err(_) => panic!(""),
        };

        loop {
            tokio::select!(
                Some(msg) = channel.1.recv() => {
                    println!("received message: {:?}", msg);
                    process_message(&conn, &msg, proxy_id).await;
                },
                _ = quit_r.recv() => {println!("terminating signal"); break;}
            );
        }
        conn.unregister_service(svc).await;
    });

    match tokio::signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }

    let _ = quit_s.send(true);
    let _ = tsk.await;

    connection.stop().await;
}

async fn process_message(conn: &Arc<Connection>, msg: &capirs::Command, proxy_id: capirs::ProxyID) {
    match msg {
        capirs::Command::ServiceAvailable(_, _) => {
            let payload = bytes::Bytes::from("Good Morning .. ");
            let result2 = conn.send_request(proxy_id, 0x1111, 0x2222, 0x0001,
                                             false, false, Some(payload)).await;
            assert!(result2.is_ok());
        },
        capirs::Command::Response(msg, payload) => {
            assert_eq!(msg.method, 0x0001);
            if let Some(data) = payload {
                println!("Response: {:?}", data);
                let result = conn.send_request(proxy_id, 0x1111,
                                               0x2222, 0x0002, false,
                                               false, None).await;
                assert!(result.is_ok());
            }
        },
        capirs::Command::Error(msg, _payload) => {
            println!("received error for method {} error {:x}", msg.method, msg.return_code.value());
        },
        _ => {},
    }
}