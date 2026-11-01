fn named(value: u16) -> u16 { return value; }
#[test]
fn stray() {}
#[cfg(test)]
mod tests {
    #[test]
    fn nested() {}
}
#[cfg(test)]
mod declared;
#[cfg(not(test))]
mod release {}
