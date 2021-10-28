use std::sync::Arc;
use capirs::*;
// use bytes::BufMut;

#[derive(Debug)]
enum MyServiceMessage {
    Request1{request: someip::Message, data: std::string::String},
    Request2{request: someip::Message, data: u32},
    Request3{request: someip::Message},
}

struct MyService {
    instance: someip::InstanceID,
    connection: Arc<capirs::Connection>,
    receiver: tokio::sync::mpsc::Receiver<someip::Command>,
    runtime: Arc<capirs::Runtime>
}

impl ServiceDescriptor for MyService {
    type StubType = Self;

    fn service_id() -> someip::ServiceID { 0x1111 }

    fn version() -> (someip::MajorVersion, someip::MinorVersion) { (1, 0) }

    fn event_descriptors(_instance: someip::InstanceID) -> std::vec::Vec<EventDescriptor> {
        vec![
            EventDescriptor { id: 0x8001, grp: 1, typ: someip::EventType::Broadcast, rel: someip::EventReliability::Service }
        ]
    }

    fn create_stub(instance: someip::InstanceID, receiver: tokio::sync::mpsc::Receiver<someip::Command>,
                   connection: Arc<Connection>, runtime: Arc<capirs::Runtime>) -> Self::StubType {
        MyService { instance, connection, receiver, runtime }
    }
}

impl MyService {
    pub async fn recv(&mut self) -> Option<MyServiceMessage> {
        match self.receiver.recv().await {
            None => { return None; },
            Some(msg) => {
                return match msg {
                    someip::Command::Request(header, payload) => self.process_request(header,  payload),
                    _ => None,
                };
            }
        }
    }

    pub async fn reply_to_request1(&self, request: &someip::Message, return_code: someip::ReturnCode,
            payload: Option<bytes::Bytes>) -> Result<(), capirs::CapiError> {
        self.connection.send_response(request, return_code, payload).await
    }

    pub async fn reply_request2(&self, request: &someip::Message, return_code: someip::ReturnCode,
            payload: Option<bytes::Bytes>) -> Result<(), capirs::CapiError> {
        self.connection.send_response(request, return_code, payload).await
    }

    pub async fn send_notification_1(&self, force: bool, payload: Option<bytes::Bytes>) {
        self.connection.send_notification(Self::service_id(), self.instance, 0x8001, payload, force).await
    }

    fn process_request(&self, header: someip::Message, payload: Option<bytes::Bytes>) -> Option<MyServiceMessage> {
        return match header.method {
            1 => {
                if let Some(data) = payload {
                    Some( MyServiceMessage::Request1{
                        request: header, data: std::str::from_utf8(&data as &[u8]).unwrap().to_string() })
                }
                else {
                    Some(MyServiceMessage::Request1{ request:header, data: std::string::String::new() })
                }
            },
            2 => Some( MyServiceMessage::Request2{request: header, data: 42 }),
            3 => Some( MyServiceMessage::Request3{request: header}),
            _ => None
        }
    }
}

impl Drop for MyService {
    fn drop(&mut self) {
        self.runtime.remove_service::<MyService>(self.instance);
    }
}


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