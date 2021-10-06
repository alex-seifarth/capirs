/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
mod vsomeip;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let rt = vsomeip::Runtime::create();
        let app = rt.create_application("my-app");


    }
}
