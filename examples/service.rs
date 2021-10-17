use capirs::*;

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
        let result = conn.register_service(svc.clone(), channel.0).await;
        assert!(result.is_ok());

        loop {
            tokio::select!(
                msg = channel.1.recv() => {
                    println!("received message: {:?}", msg);
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
//    thr.join().unwrap();
}