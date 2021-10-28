/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
mod vsomeipc;

mod runtime;
mod connection;
mod types;

pub mod someip {
    pub use super::types::*;
}

pub use connection::*;
pub use runtime::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use tokio;

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
