/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
mod vsomeip;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let rt = vsomeip::Runtime::create();
        let app = Arc::new(rt.create_application("my-app").unwrap());

        println!("main thread: {:?}", std::thread::current().id());
        let app1 = app.clone();
        app.register_state_handler(move |state: vsomeip::AppRegistrationState| {
            println!("application state: {:?} [{}]", state, app1.name());
            println!("state handler thread: {:?}", std::thread::current().id());
            app1.offer_service(0x1111, 0x2222, vsomeip::DEFAULT_MAJOR, vsomeip::DEFAULT_MINOR);
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
}
