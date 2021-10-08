/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
mod vsomeipc;
mod message;
mod vsomeip;

mod connection;
mod service_adapter;
mod types;

mod someip {
    pub use super::types::*;
    pub use super::service_adapter::*;
}

pub use connection::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use tokio;

    #[test]
    #[ignore]
    fn it_works() {
        let rt = vsomeip::Runtime::create();
        let app = rt.create_application("my-app").unwrap();

        println!("main thread: {:?}", std::thread::current().id());
        let app1 = app.clone();
        app.register_state_handler(move |state: vsomeip::AppRegistrationState| {
            println!("application state: {:?} [{}]", state, app1.name());
            println!("state handler thread: {:?}", std::thread::current().id());
            app1.offer_service(0x1111, 0x2222, message::DEFAULT_MAJOR, message::DEFAULT_MINOR);
        });

        let appc = app.clone();
        let thr = std::thread::spawn(move || {
            println!("start thread: {:?}", std::thread::current().id());
            appc.start();
            println!("application finished");
        });

        thread::sleep(std::time::Duration::from_secs(2));
        app.stop();
        thr.join().unwrap();
    }

    #[test]
    fn test_connection() {
        let connection = connection::Connection::create("connection-app").unwrap();

        let thr = connection.start(true);

        let channel = tokio::sync::mpsc::channel(1024);
        let svc = someip::ServiceInstanceID{
            service: 0x1111, instance: 0x2222, major_version: 0x01, minor_version: someip::DEFAULT_MINOR
        };

        let result = connection.register_service(svc.clone(), channel.0);
        assert!(result.is_ok());
        thread::sleep(std::time::Duration::from_secs(2));

        connection.unregister_service(svc);
        thread::sleep(std::time::Duration::from_secs(1));

        connection.stop();
        thr.join().unwrap();

    }
}
