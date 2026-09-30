use myrix_chain::{consensus::Consensus, core::{Block, Ledger}, validator::{ValidatorInfo, ValidatorSet}};

#[test]
fn validator_leader_is_deterministic() {
    let mut set = ValidatorSet::new();
    set.add_validator(ValidatorInfo::new("validator-a".into(), "00".into(), 100)).unwrap();
    set.add_validator(ValidatorInfo::new("validator-b".into(), "00".into(), 200)).unwrap();
    assert_eq!(set.get_leader(0), set.get_leader(0));
    assert!(set.contains_active("validator-a"));
}

#[test]
fn quorum_uses_stake() {
    let mut set = ValidatorSet::new();
    set.add_validator(ValidatorInfo::new("a".into(), "00".into(), 100)).unwrap();
    set.add_validator(ValidatorInfo::new("b".into(), "00".into(), 100)).unwrap();
    set.add_validator(ValidatorInfo::new("c".into(), "00".into(), 100)).unwrap();
    assert_eq!(set.quorum(), 201);
    assert!(!Consensus::new().has_quorum(&set, &["a".into(), "b".into()]));
    assert!(Consensus::new().has_quorum(&set, &["a".into(), "b".into(), "c".into()]));
}

#[test]
fn empty_block_has_valid_hash_and_chain_link() {
    let mut ledger = Ledger::new();
    let mut block = Block {
        index: 1,
        prev_hash: "0".into(),
        transactions: vec![],
        timestamp: 1,
        proposer: "validator-a".into(),
        hash: String::new(),
        gas_used: 0,
    };
    block.hash = block.compute_hash();
    assert!(block.validate_header(1, "0").is_ok());
    assert!(ledger.apply_block(&block).is_ok());
    assert_eq!(ledger.height, 1);
    assert_eq!(ledger.last_hash, block.hash);
}


#[test]
fn signed_transaction_round_trip_is_valid() {
    use myrix_chain::wallet::Wallet;
    use myrix_chain::core::{Account, Ledger, Transaction};
    let wallet = Wallet::generate();
    let receiver = "0xreceiver".to_string();
    let mut ledger = Ledger::new();
    ledger.accounts.insert(wallet.address.clone(), Account { id: wallet.address.clone(), balance: 1_000, nonce: 0 });
    let mut tx = Transaction {
        sender: wallet.address.clone(),
        receiver,
        amount: 100,
        nonce: 0,
        fee: 1,
        public_key: wallet.public_key().to_string(),
        signature: String::new(),
        hash: String::new(),
    };
    tx.signature = wallet.sign(&tx.signing_bytes());
    tx.hash = tx.compute_hash();
    assert!(tx.validate(ledger.accounts.get(&tx.sender)).is_ok());
    assert!(ledger.apply_transaction(&tx).is_ok());
    assert_eq!(ledger.accounts[&tx.sender].balance, 899);
}
