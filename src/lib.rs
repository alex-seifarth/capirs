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
