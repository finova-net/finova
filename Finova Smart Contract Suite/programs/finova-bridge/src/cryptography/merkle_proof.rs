// programs/finova-bridge/src/cryptography/merkle_proof.rs

use anchor_lang::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use crate::errors::BridgeError;

/// Maximum depth for Merkle tree to prevent DoS attacks
pub const MAX_MERKLE_DEPTH: usize = 32;

/// Minimum number of leaves required for a valid Merkle tree
pub const MIN_MERKLE_LEAVES: usize = 2;

/// Maximum number of proofs that can be verified in a single transaction
pub const MAX_BATCH_PROOFS: usize = 10;

/// Standard leaf prefix to prevent second preimage attacks
pub const LEAF_PREFIX: &[u8] = b"FINOVA_LEAF";

/// Standard node prefix to prevent second preimage attacks  
pub const NODE_PREFIX: &[u8] = b"FINOVA_NODE";

/// Merkle tree node representation
#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub is_leaf: bool,
    pub data: Option<Vec<u8>>,
}

impl MerkleNode {
    /// Create a new leaf node with data
    pub fn new_leaf(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(LEAF_PREFIX);
        hasher.update(data);
        let hash = hasher.finalize().into();
        
        Self {
            hash,
            is_leaf: true,
            data: Some(data.to_vec()),
        }
    }

    /// Create a new internal node from two child nodes
    pub fn new_internal(left: &[u8; 32], right: &[u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(NODE_PREFIX);
        hasher.update(left);
        hasher.update(right);
        let hash = hasher.finalize().into();
        
        Self {
            hash,
            is_leaf: false,
            data: None,
        }
    }

    /// Get the hash of this node
    pub fn get_hash(&self) -> [u8; 32] {
        self.hash
    }
}

/// Merkle proof path element
#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct MerkleProofElement {
    pub hash: [u8; 32],
    pub is_left: bool, // true if this element is the left sibling
}

/// Complete Merkle proof structure
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MerkleProof {
    pub leaf_hash: [u8; 32],
    pub leaf_index: u64,
    pub proof_elements: Vec<MerkleProofElement>,
    pub root_hash: [u8; 32],
    pub tree_size: u64,
}

impl MerkleProof {
    /// Create a new Merkle proof
    pub fn new(
        leaf_hash: [u8; 32],
        leaf_index: u64,
        proof_elements: Vec<MerkleProofElement>,
        root_hash: [u8; 32],
        tree_size: u64,
    ) -> Result<Self, BridgeError> {
        // Validate proof depth
        if proof_elements.len() > MAX_MERKLE_DEPTH {
            return Err(BridgeError::InvalidMerkleProofDepth);
        }

        // Validate tree size
        if tree_size < MIN_MERKLE_LEAVES as u64 {
            return Err(BridgeError::InvalidMerkleTreeSize);
        }

        // Validate leaf index
        if leaf_index >= tree_size {
            return Err(BridgeError::InvalidMerkleLeafIndex);
        }

        Ok(Self {
            leaf_hash,
            leaf_index,
            proof_elements,
            root_hash,
            tree_size,
        })
    }

    /// Verify the Merkle proof
    pub fn verify(&self) -> Result<bool, BridgeError> {
        if self.proof_elements.is_empty() {
            return Ok(self.leaf_hash == self.root_hash && self.tree_size == 1);
        }

        let mut current_hash = self.leaf_hash;
        let mut current_index = self.leaf_index;

        for element in &self.proof_elements {
            // Determine if current node is left or right child
            let is_current_left = current_index % 2 == 0;
            
            // Calculate parent hash based on position
            current_hash = if is_current_left {
                if element.is_left {
                    return Err(BridgeError::InvalidMerkleProofStructure);
                }
                // Current is left, element is right
                Self::hash_pair(&current_hash, &element.hash)
            } else {
                if !element.is_left {
                    return Err(BridgeError::InvalidMerkleProofStructure);
                }
                // Element is left, current is right
                Self::hash_pair(&element.hash, &current_hash)
            };

            // Move to parent index
            current_index /= 2;
        }

        Ok(current_hash == self.root_hash)
    }

    /// Hash two nodes together to create parent node hash
    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(NODE_PREFIX);
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    /// Get the depth of the proof
    pub fn depth(&self) -> usize {
        self.proof_elements.len()
    }

    /// Validate proof format and structure
    pub fn validate_format(&self) -> Result<(), BridgeError> {
        // Check depth limits
        if self.proof_elements.len() > MAX_MERKLE_DEPTH {
            return Err(BridgeError::InvalidMerkleProofDepth);
        }

        // Check tree size
        if self.tree_size < MIN_MERKLE_LEAVES as u64 {
            return Err(BridgeError::InvalidMerkleTreeSize);
        }

        // Check leaf index
        if self.leaf_index >= self.tree_size {
            return Err(BridgeError::InvalidMerkleLeafIndex);
        }

        // Calculate expected depth
        let expected_depth = (64 - (self.tree_size - 1).leading_zeros()) as usize;
        if self.proof_elements.len() != expected_depth {
            return Err(BridgeError::InvalidMerkleProofDepth);
        }

        Ok(())
    }
}

/// Merkle tree builder for creating trees and generating proofs
pub struct MerkleTreeBuilder {
    leaves: Vec<MerkleNode>,
    tree_levels: Vec<Vec<MerkleNode>>,
}

impl MerkleTreeBuilder {
    /// Create a new Merkle tree builder
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            tree_levels: Vec::new(),
        }
    }

    /// Add a leaf to the tree
    pub fn add_leaf(&mut self, data: &[u8]) -> Result<(), BridgeError> {
        if self.leaves.len() >= (1 << MAX_MERKLE_DEPTH) {
            return Err(BridgeError::MerkleTreeTooLarge);
        }
        
        self.leaves.push(MerkleNode::new_leaf(data));
        Ok(())
    }

    /// Build the complete Merkle tree
    pub fn build(&mut self) -> Result<[u8; 32], BridgeError> {
        if self.leaves.len() < MIN_MERKLE_LEAVES {
            return Err(BridgeError::InvalidMerkleTreeSize);
        }

        self.tree_levels.clear();
        self.tree_levels.push(self.leaves.clone());

        let mut current_level = 0;
        
        while self.tree_levels[current_level].len() > 1 {
            let mut next_level = Vec::new();
            let current_nodes = &self.tree_levels[current_level];
            
            for i in (0..current_nodes.len()).step_by(2) {
                let left = &current_nodes[i];
                let right = if i + 1 < current_nodes.len() {
                    &current_nodes[i + 1]
                } else {
                    // Duplicate last node for odd number of nodes
                    &current_nodes[i]
                };
                
                let parent = MerkleNode::new_internal(&left.hash, &right.hash);
                next_level.push(parent);
            }
            
            self.tree_levels.push(next_level);
            current_level += 1;
        }

        Ok(self.tree_levels[current_level][0].hash)
    }

    /// Generate a Merkle proof for a specific leaf index
    pub fn generate_proof(&self, leaf_index: usize) -> Result<MerkleProof, BridgeError> {
        if self.tree_levels.is_empty() {
            return Err(BridgeError::MerkleTreeNotBuilt);
        }

        if leaf_index >= self.leaves.len() {
            return Err(BridgeError::InvalidMerkleLeafIndex);
        }

        let mut proof_elements = Vec::new();
        let mut current_index = leaf_index;
        let leaf_hash = self.leaves[leaf_index].hash;
        
        // Traverse from leaf to root, collecting sibling hashes
        for level in 0..(self.tree_levels.len() - 1) {
            let level_nodes = &self.tree_levels[level];
            let sibling_index = if current_index % 2 == 0 {
                // Current is left child, sibling is right
                if current_index + 1 < level_nodes.len() {
                    current_index + 1
                } else {
                    current_index // Duplicate for odd number of nodes
                }
            } else {
                // Current is right child, sibling is left
                current_index - 1
            };

            let is_sibling_left = sibling_index < current_index;
            proof_elements.push(MerkleProofElement {
                hash: level_nodes[sibling_index].hash,
                is_left: is_sibling_left,
            });

            current_index /= 2;
        }

        let root_hash = self.tree_levels.last().unwrap()[0].hash;
        
        MerkleProof::new(
            leaf_hash,
            leaf_index as u64,
            proof_elements,
            root_hash,
            self.leaves.len() as u64,
        )
    }

    /// Get the root hash of the built tree
    pub fn get_root(&self) -> Result<[u8; 32], BridgeError> {
        if self.tree_levels.is_empty() {
            return Err(BridgeError::MerkleTreeNotBuilt);
        }
        
        Ok(self.tree_levels.last().unwrap()[0].hash)
    }

    /// Get the total number of leaves
    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }
}

/// Batch verification of multiple Merkle proofs
pub struct BatchMerkleVerifier {
    proofs: Vec<MerkleProof>,
    cache: HashMap<[u8; 32], bool>,
}

impl BatchMerkleVerifier {
    /// Create a new batch verifier
    pub fn new() -> Self {
        Self {
            proofs: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// Add a proof to the batch
    pub fn add_proof(&mut self, proof: MerkleProof) -> Result<(), BridgeError> {
        if self.proofs.len() >= MAX_BATCH_PROOFS {
            return Err(BridgeError::TooManyProofsInBatch);
        }
        
        proof.validate_format()?;
        self.proofs.push(proof);
        Ok(())
    }

    /// Verify all proofs in the batch
    pub fn verify_all(&mut self) -> Result<Vec<bool>, BridgeError> {
        let mut results = Vec::new();
        
        for proof in &self.proofs {
            // Check cache first
            let cache_key = proof.leaf_hash;
            if let Some(&cached_result) = self.cache.get(&cache_key) {
                results.push(cached_result);
                continue;
            }

            // Verify proof
            let is_valid = proof.verify()?;
            
            // Cache result
            self.cache.insert(cache_key, is_valid);
            results.push(is_valid);
        }

        Ok(results)
    }

    /// Clear all proofs and cache
    pub fn clear(&mut self) {
        self.proofs.clear();
        self.cache.clear();
    }

    /// Get number of proofs in batch
    pub fn proof_count(&self) -> usize {
        self.proofs.len()
    }
}

/// Incremental Merkle tree for efficient updates
pub struct IncrementalMerkleTree {
    nodes: Vec<Option<[u8; 32]>>,
    depth: usize,
    next_index: u64,
}

impl IncrementalMerkleTree {
    /// Create a new incremental Merkle tree
    pub fn new(depth: usize) -> Result<Self, BridgeError> {
        if depth > MAX_MERKLE_DEPTH {
            return Err(BridgeError::InvalidMerkleProofDepth);
        }

        let total_nodes = (1 << (depth + 1)) - 1;
        Ok(Self {
            nodes: vec![None; total_nodes],
            depth,
            next_index: 0,
        })
    }

    /// Insert a new leaf and return the updated root
    pub fn insert(&mut self, leaf_data: &[u8]) -> Result<[u8; 32], BridgeError> {
        if self.next_index >= (1 << self.depth) {
            return Err(BridgeError::MerkleTreeFull);
        }

        let leaf_hash = {
            let mut hasher = Sha256::new();
            hasher.update(LEAF_PREFIX);
            hasher.update(leaf_data);
            hasher.finalize().into()
        };

        // Insert leaf at the correct position
        let leaf_position = ((1 << self.depth) - 1) + self.next_index as usize;
        self.nodes[leaf_position] = Some(leaf_hash);
        
        // Update path to root
        self.update_path(leaf_position)?;
        
        self.next_index += 1;
        self.get_root()
    }

    /// Update the path from a leaf to the root
    fn update_path(&mut self, mut position: usize) -> Result<(), BridgeError> {
        while position > 0 {
            let parent_pos = (position - 1) / 2;
            let left_child_pos = parent_pos * 2 + 1;
            let right_child_pos = parent_pos * 2 + 2;

            // Get child hashes
            let left_hash = self.nodes.get(left_child_pos)
                .and_then(|node| *node)
                .unwrap_or([0u8; 32]);
            
            let right_hash = self.nodes.get(right_child_pos)
                .and_then(|node| *node)
                .unwrap_or([0u8; 32]);

            // Calculate parent hash
            let parent_hash = if left_hash == [0u8; 32] && right_hash == [0u8; 32] {
                [0u8; 32]
            } else {
                let mut hasher = Sha256::new();
                hasher.update(NODE_PREFIX);
                hasher.update(&left_hash);
                hasher.update(&right_hash);
                hasher.finalize().into()
            };

            self.nodes[parent_pos] = if parent_hash == [0u8; 32] { None } else { Some(parent_hash) };
            position = parent_pos;
        }

        Ok(())
    }

    /// Get the current root of the tree
    pub fn get_root(&self) -> Result<[u8; 32], BridgeError> {
        self.nodes[0].ok_or(BridgeError::MerkleTreeEmpty)
    }

    /// Generate a proof for a specific leaf index
    pub fn generate_proof(&self, leaf_index: u64) -> Result<MerkleProof, BridgeError> {
        if leaf_index >= self.next_index {
            return Err(BridgeError::InvalidMerkleLeafIndex);
        }

        let mut proof_elements = Vec::new();
        let leaf_position = ((1 << self.depth) - 1) + leaf_index as usize;
        let leaf_hash = self.nodes[leaf_position].ok_or(BridgeError::MerkleLeafNotFound)?;
        
        let mut current_pos = leaf_position;
        
        while current_pos > 0 {
            let parent_pos = (current_pos - 1) / 2;
            let sibling_pos = if current_pos % 2 == 1 {
                // Current is left child, sibling is right
                current_pos + 1
            } else {
                // Current is right child, sibling is left  
                current_pos - 1
            };

            let sibling_hash = self.nodes.get(sibling_pos)
                .and_then(|node| *node)
                .unwrap_or([0u8; 32]);

            let is_sibling_left = sibling_pos < current_pos;
            proof_elements.push(MerkleProofElement {
                hash: sibling_hash,
                is_left: is_sibling_left,
            });

            current_pos = parent_pos;
        }

        let root_hash = self.get_root()?;
        
        MerkleProof::new(
            leaf_hash,
            leaf_index,
            proof_elements,
            root_hash,
            self.next_index,
        )
    }

    /// Get the number of leaves inserted
    pub fn leaf_count(&self) -> u64 {
        self.next_index
    }

    /// Check if the tree is full
    pub fn is_full(&self) -> bool {
        self.next_index >= (1 << self.depth)
    }
}

/// Utilities for Merkle proof operations
pub mod utils {
    use super::*;

    /// Convert bytes to hex string for debugging
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Create a simple Merkle tree from data array
    pub fn create_simple_tree(data_items: &[&[u8]]) -> Result<([u8; 32], Vec<MerkleProof>), BridgeError> {
        let mut builder = MerkleTreeBuilder::new();
        
        for item in data_items {
            builder.add_leaf(item)?;
        }
        
        let root = builder.build()?;
        let mut proofs = Vec::new();
        
        for i in 0..data_items.len() {
            proofs.push(builder.generate_proof(i)?);
        }
        
        Ok((root, proofs))
    }

    /// Verify a batch of proofs against the same root
    pub fn batch_verify_same_root(proofs: &[MerkleProof], expected_root: &[u8; 32]) -> Result<bool, BridgeError> {
        for proof in proofs {
            if proof.root_hash != *expected_root {
                return Ok(false);
            }
            if !proof.verify()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_node_creation() {
        let data = b"test_data";
        let leaf = MerkleNode::new_leaf(data);
        assert!(leaf.is_leaf);
        assert_eq!(leaf.data, Some(data.to_vec()));
        
        let left_hash = [1u8; 32];
        let right_hash = [2u8; 32];
        let internal = MerkleNode::new_internal(&left_hash, &right_hash);
        assert!(!internal.is_leaf);
        assert!(internal.data.is_none());
    }

    #[test]
    fn test_simple_merkle_tree() {
        let mut builder = MerkleTreeBuilder::new();
        builder.add_leaf(b"leaf1").unwrap();
        builder.add_leaf(b"leaf2").unwrap();
        builder.add_leaf(b"leaf3").unwrap();
        builder.add_leaf(b"leaf4").unwrap();
        
        let root = builder.build().unwrap();
        assert_ne!(root, [0u8; 32]);
        
        let proof = builder.generate_proof(0).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_incremental_merkle_tree() {
        let mut tree = IncrementalMerkleTree::new(3).unwrap();
        
        let root1 = tree.insert(b"data1").unwrap();
        let root2 = tree.insert(b"data2").unwrap();
        
        assert_ne!(root1, root2);
        assert_eq!(tree.leaf_count(), 2);
        
        let proof = tree.generate_proof(0).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_batch_verification() {
        let data = [b"item1".as_slice(), b"item2", b"item3", b"item4"];
        let (root, proofs) = utils::create_simple_tree(&data).unwrap();
        
        let mut verifier = BatchMerkleVerifier::new();
        for proof in proofs {
            verifier.add_proof(proof).unwrap();
        }
        
        let results = verifier.verify_all().unwrap();
        assert!(results.iter().all(|&r| r));
    }
}
