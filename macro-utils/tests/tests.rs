#[cfg(test)]
mod tests {

    #[derive(macro_utils::TryFrom, PartialEq, Debug)]
    #[conversion_type(i64)]
    enum Example{
        one,
        two,
        three
    }

    #[test]
    fn simple_conversion() {
        let mut e: Example = 1.try_into().unwrap();
        assert_eq!(e, Example::two);
    }
}