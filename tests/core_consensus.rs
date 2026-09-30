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
        round: 0,
        chain_id: "test-chain".into(),
        prev_hash: "0".into(),
        transactions: vec![],
        timestamp: 1,
        proposer: "validator-a".into(),
        proposer_signature: "test-signature".into(),
        hash: String::new(),
        gas_used: 0,
    };
    block.hash = block.compute_hash();
    assert!(block.validate_header(1, "0", "test-chain").is_ok());
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


#[test]
fn signed_votes_reach_stake_quorum_and_certificate() {
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
    use myrix_chain::consensus::Vote;

    let keys = [
        SigningKey::from_bytes(&[1u8; 32]),
        SigningKey::from_bytes(&[2u8; 32]),
        SigningKey::from_bytes(&[3u8; 32]),
    ];
    let mut set = ValidatorSet::new();
    for (i, key) in keys.iter().enumerate() {
        set.add_validator(ValidatorInfo::new(
            format!("validator-{}", i + 1),
            format!("ed25519:{}", hex::encode(VerifyingKey::from(key).to_bytes())),
            100,
        )).unwrap();
    }

    let mut block = Block {
        index: 1,
        round: 0,
        chain_id: "test-chain".into(),
        prev_hash: "0".into(),
        transactions: vec![],
        timestamp: 1,
        proposer: "validator-1".into(),
        proposer_signature: String::new(),
        hash: String::new(),
        gas_used: 0,
    };
    block.hash = block.compute_hash();
    block.proposer_signature = hex::encode(keys[0].sign(&block.signing_bytes()).to_bytes());

    let mut votes = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        let bytes = Consensus::vote_signing_bytes("test-chain", 1, 0, &block.hash);
        votes.push(Vote {
            validator_id: format!("validator-{}", i + 1),
            chain_id: "test-chain".into(),
            height: 1,
            round: 0,
            block_hash: block.hash.clone(),
            signature: hex::encode(key.sign(&bytes).to_bytes()),
        });
    }

    let consensus = Consensus::new();
    let cert = consensus.build_certificate(&block, &votes, &set).unwrap();
    assert_eq!(cert.voters.len(), 3);
    assert!(consensus.verify_certificate(&block, &cert, &set).is_ok());
}
