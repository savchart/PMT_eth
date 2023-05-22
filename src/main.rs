use std::fs;
use std::sync::Arc;
use eth_trie::{EthTrie, Trie, TrieError, MemoryDB};
use ethers::abi::AbiEncode;
use ethers::types::{Transaction};
use ethers::utils::{rlp};


fn load_transactions_from_json(file_path: &str) -> Vec<Transaction> {
    let data = fs::read_to_string(file_path).expect("Unable to read file");
    serde_json::from_str::<Vec<Transaction>>(&data).expect("JSON was not well-formatted")
}

fn main() -> Result<(), TrieError> {
    let memdb = Arc::new(MemoryDB::new(true));
    let file_path = "block_10593417.json";
    let transactions = load_transactions_from_json(file_path);

    let mut trie = EthTrie::new(memdb.clone());
    for (i, transaction) in transactions.iter().enumerate() {

        let mut stream = rlp::RlpStream::new();
        stream.append(&i);
        let key = stream.out();

        let value = transaction.rlp();
        trie.insert(&key, &value)?;
    }

    let root = trie.root_hash()?;

    // verify root hash (compute vs in the block header)
    // get from etherscan api
    let expected_root = "ab41f886be23cd786d8a69a72b0f988ea72e0b2e03970d0798f5e03763a442cc";
    assert_eq!(format!("{:x}", root), expected_root, "Root hashes do not match");
    println!("Hashes match!");
    // generate proof for a transaction
    let mut key_stream = rlp::RlpStream::new();
    key_stream.append(&0u16); // For the first transaction
    let key = key_stream.out();
    let proof = trie.get_proof(&key)?;

    // verify proof
    let verification_result = trie.verify_proof(root, &key, proof)?;
    match verification_result {
        Some(value) => println!("Key exists, value: {:?}", value),
        None => println!("Key does not exist"),
    }
    // verify proof for a non-existent key
    let mut key_stream = rlp::RlpStream::new();
    key_stream.append(&100u16); // For the first transaction
    let key = key_stream.out();
    let proof = trie.get_proof(&key)?;
    let verification_result = trie.verify_proof(root, &key, proof)?;
    match verification_result {
        Some(value) => println!("Key exists, value: {:?}", value.encode_hex()),
        None => println!("Key does not exist"),
    }

    Ok(())
}
