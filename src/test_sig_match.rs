use tiny_keccak::Keccak;
use tiny_keccak::Hasher;
use hex_literal::hex;

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut keccak = Keccak::v256();
    keccak.update(data);
    let mut output = [0u8; 32];
    keccak.finalize(&mut output);
    output
}

pub fn test_hash() {
    let signature = "Swap(address,address,uint256,uint256,uint256,uint256,uint256)";
    let computed_hash = keccak256(signature.as_bytes());
    let expected_hash = hex!("c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67");
    println!("Computed hash: {:x?}", computed_hash);
    println!("Expected hash: {:x?}", expected_hash);
    assert_eq!(computed_hash, expected_hash);
}
