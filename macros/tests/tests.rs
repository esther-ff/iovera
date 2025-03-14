extern crate iovera_macros;

#[iovera_macros::parse_struct]
pub struct TestStruct<'lf, T, I: Iterator> {
    a: i32,
    b: u32,
    c: f32,
    mut_lifetime_generic: &'lf mut T,
    generic: T,
    generic_boxed: Box<T>,
    generic_boxed_iterator: Box<I>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        //let teststruct = tests::TestStruct::new(1, 2, 3.0);

        //println!("{:#?}", teststruct);
    }
}
