#![no_std]

use defmt::*;

pub fn add(left: u64, right: u64) -> u64 {
    info!("in fn add");
    left + right
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
