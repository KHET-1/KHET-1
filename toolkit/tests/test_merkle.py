"""Test suite for Merkle Tree module."""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from merkle.tree import MerkleTree


def test_basic_tree():
    tree = MerkleTree()
    tree.add_leaf("evidence_file_1")
    tree.add_leaf("evidence_file_2")
    tree.add_leaf("evidence_file_3")

    root = tree.build()
    assert root, "Root should not be empty"
    assert len(root) == 64, "SHA-256 produces 64 hex chars"
    print(f"[PASS] Basic tree: root={root[:24]}...")


def test_deterministic():
    tree1 = MerkleTree()
    tree1.add_leaf("a")
    tree1.add_leaf("b")
    root1 = tree1.build()

    tree2 = MerkleTree()
    tree2.add_leaf("a")
    tree2.add_leaf("b")
    root2 = tree2.build()

    assert root1 == root2, "Same inputs must produce same root"
    print(f"[PASS] Deterministic: both roots = {root1[:24]}...")


def test_different_data_different_root():
    tree1 = MerkleTree()
    tree1.add_leaf("a")
    tree1.add_leaf("b")
    root1 = tree1.build()

    tree2 = MerkleTree()
    tree2.add_leaf("a")
    tree2.add_leaf("c")
    root2 = tree2.build()

    assert root1 != root2, "Different inputs must produce different roots"
    print(f"[PASS] Divergent roots confirmed")


def test_proof_generation():
    tree = MerkleTree()
    tree.add_leaf("alpha")
    tree.add_leaf("beta")
    tree.add_leaf("gamma")
    tree.add_leaf("delta")
    tree.build()

    proof = tree.get_proof(0)
    assert len(proof) > 0, "Proof should have at least one element"
    print(f"[PASS] Proof for leaf 0: {len(proof)} steps")


def test_manifest():
    tree = MerkleTree()
    tree.add_leaf("file_hash_1", label="evidence:capture.pcap")
    tree.add_leaf("file_hash_2", label="evidence:trace.pcapng")
    tree.build()

    manifest = tree.to_manifest()
    assert manifest['merkle_root'] == tree.get_root()
    assert manifest['leaf_count'] == 2
    assert manifest['algorithm'] == 'SHA-256'
    assert len(manifest['leaves']) == 2
    assert manifest['leaves'][0]['label'] == 'evidence:capture.pcap'
    print(f"[PASS] Manifest: {manifest['leaf_count']} leaves, root={manifest['merkle_root'][:16]}...")


def test_single_leaf():
    tree = MerkleTree()
    tree.add_leaf("only_one")
    root = tree.build()
    assert root, "Single leaf tree should have a root"
    print(f"[PASS] Single leaf: root={root[:24]}...")


if __name__ == '__main__':
    test_basic_tree()
    test_deterministic()
    test_different_data_different_root()
    test_proof_generation()
    test_manifest()
    test_single_leaf()
    print("\n[ALL TESTS PASSED]")
