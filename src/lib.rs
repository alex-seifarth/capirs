/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
mod vsomeip;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn it_works() {
        let rt = vsomeip::Runtime::create();
        let app = rt.create_application("my-app").unwrap();
        let appc = app.clone();

        let thr = std::thread::spawn(move || {
            appc.start();
            println!("application finished");
        });

        thread::sleep(std::time::Duration::from_secs(1));
        app.stop();
        thr.join();
    }
}
