use capirs::*;
mod defs;
use defs::*;

// use bytes::BufMut;

#[tokio::main]
pub async fn main() {
    let runtime = Runtime::create("service2").await;
    let mut my_service_stub = match runtime.create_service::<MyService>(0x2222).await {
        Ok(stub) => stub,
        Err(err) => { panic!("Cannot create stub: {:?}", err); }
    };

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            },
            Some(msg) = my_service_stub.recv() => {
                match msg {
                    MyServiceMessage::Request1{request, data} => {
                        println!("Request1 [{:?}]", data);
                        my_service_stub.reply_to_request1(&request, someip::ReturnCode::Ok,
                            Some(bytes::Bytes::from("F.ck the morning .. "))).await.unwrap();
                    },
                    MyServiceMessage::Request2{request, data} => {
                        println!("Request2 [{:?}]", data);
                        my_service_stub.reply_request2(&request, someip::ReturnCode::ApplicationError(22),
                            None).await.unwrap();
                    },
                    MyServiceMessage::Request3{request:_} => {
                        // don't react -> client should get a timeout
                        // let mut payload = bytes::BytesMut::new();
                        // payload.put_u32(0x4711);
                        my_service_stub.send_notification_1(false, None).await; //Some(payload.freeze())).await;
                    }
                }

            },
        };
    }

    drop(my_service_stub);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}