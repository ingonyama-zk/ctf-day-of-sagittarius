use sagittarius_game::{state::GameState, abilities::{ShotParams, ShotCommit, ClusterBombParams, ClusterCommit, ScoutParams, ScoutResult}, types::{Position, Digest, HitType}};
use sagittarius_methods::{INIT_ID, INIT_ELF, TURN_ID, TURN_ELF, CLUSTER_ID, CLUSTER_ELF, SCOUT_ELF, SCOUT_ID};
use risc0_zkvm::{Result, Receipt, serde, Prover};

// Init

pub fn create_init_proof(input: &GameState) -> Result<Receipt> {
    let mut prover = Prover::new(&INIT_ELF)?;
    let vec = serde::to_vec(&input).unwrap();
    prover.add_input_u32_slice(vec.as_slice());
    prover.run()
}

pub fn check_init_proof(receipt: &Receipt) -> Result<()> {
    receipt.verify(&INIT_ID)
}

// Simple shot proof

pub fn create_turn_proof(input: &ShotParams) -> Result<Receipt> {
    let mut prover = Prover::new(&TURN_ELF)?;
    let vec = serde::to_vec(&input).unwrap();
    prover.add_input_u32_slice(vec.as_slice());
    prover.run()
}

pub fn check_turn_proof(receipt: Receipt, shot: &Position, old_state: &Digest) -> Result<(HitType, Digest)> {
    receipt.verify(&TURN_ID)?;
    let journal = receipt.get_journal_bytes(); 
    let commit = serde::from_slice::<ShotCommit, u8>(&journal).unwrap();
    // Make sure the prior state matches the current state
    assert!(old_state == &commit.old_state_digest);
    // Make sure the response matches the prior shot
    assert!(commit.shot == shot.clone());
    
    Ok((commit.hit, commit.new_state_digest))
}

// Scout
pub fn create_scout_proof(input: &ScoutParams) -> Result<Receipt> {
    let mut prover = Prover::new(&SCOUT_ELF)?;
    let vec = serde::to_vec(&input).unwrap();
    prover.add_input_u32_slice(vec.as_slice());
    prover.run()
}

pub fn check_scout_proof(receipt: Receipt, shot: &Position) -> Result<Vec<HitType>> {
    receipt.verify(&SCOUT_ID)?;
    let journal = receipt.get_journal_bytes(); 
    let results = serde::from_slice::<ScoutResult, u8>(&journal).unwrap();

    assert!(&results.shot == shot);
    Ok(results.cells.to_vec())
}

// Cluster bomb proof

pub fn create_cluster_proof(input: &ClusterBombParams) -> Result<(Receipt, Vec<Position>)> {
    let mut prover = Prover::new(&CLUSTER_ELF)?;
    let vec = serde::to_vec(&input).unwrap();
    prover.add_input_u32_slice(vec.as_slice());
    let receipt = prover.run()?;

    let journal = receipt.get_journal_bytes(); 
    let commit = serde::from_slice::<ClusterCommit, u8>(&journal).unwrap();

    Ok((receipt, commit.shots))
}

pub fn check_cluster_proof(receipt: Receipt, ul: Position, dr: Position, seed: u8, old_state: Digest) -> Result<(Vec<Position>, Vec<HitType>, Digest)> {
    receipt.verify(&CLUSTER_ID)?;
    let journal = receipt.get_journal_bytes(); 
    let commit = serde::from_slice::<ClusterCommit, u8>(&journal).unwrap();
    // Make sure the prior state matches the current state
    assert!(old_state == commit.old_state_digest);
    // Make sure the response matches the Cluster config
    assert!(commit.config.upper_left_coordinates == ul);
    assert!(commit.config.down_right_coordinates == dr);
    assert!(commit.config.seed == seed);
    
    Ok((commit.shots, commit.hits, commit.new_state_digest))
}