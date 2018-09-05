#![crate_type="lib"]

pub fn some_lib_func() {}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        use super::some_lib_func;
        assert!(some_lib_func() == ())
    }
}
