import hashlib
import json
import os
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass, field
from datetime import datetime, timezone


@dataclass
class MerkleNode:
    hash: str
    left: Optional['MerkleNode'] = None
    right: Optional['MerkleNode'] = None
    data: Optional[str] = None
    label: str = ''


class MerkleTree:
    """Production-grade SHA-256 Merkle Tree with proof generation, verification, and serialization."""

    def __init__(self):
        self.leaves: List[MerkleNode] = []
        self.root: Optional[MerkleNode] = None
        self._built = False

    def add_leaf(self, data: str, label: str = '') -> str:
        """Add a data leaf. Returns leaf hash."""
        leaf_hash = hashlib.sha256(data.encode('utf-8')).hexdigest()
        node = MerkleNode(hash=leaf_hash, data=data, label=label)
        self.leaves.append(node)
        self._built = False
        return leaf_hash

    def add_file(self, filepath: str, label: str = '') -> str:
        """Add a file as a leaf (hash the file contents)."""
        data = open(filepath, 'rb').read()
        leaf_hash = hashlib.sha256(data).hexdigest()
        node = MerkleNode(hash=leaf_hash, data=None, label=label or os.path.basename(filepath))
        self.leaves.append(node)
        self._built = False
        return leaf_hash

    def add_json(self, obj, label: str = '') -> str:
        """Add a JSON-serializable object as a leaf."""
        serialized = json.dumps(obj, sort_keys=True, default=str)
        return self.add_leaf(serialized, label)

    def build(self) -> str:
        """Build the tree from current leaves. Returns root hash."""
        if not self.leaves:
            self.root = MerkleNode(hash=hashlib.sha256(b'empty').hexdigest())
            return self.root.hash

        nodes = list(self.leaves)
        while len(nodes) > 1:
            new_level = []
            for i in range(0, len(nodes), 2):
                left = nodes[i]
                right = nodes[i + 1] if i + 1 < len(nodes) else left
                combined = (left.hash + right.hash).encode('utf-8')
                parent_hash = hashlib.sha256(combined).hexdigest()
                parent = MerkleNode(hash=parent_hash, left=left, right=right)
                new_level.append(parent)
            nodes = new_level

        self.root = nodes[0]
        self._built = True
        return self.root.hash

    def get_root(self) -> str:
        """Get root hash (builds tree if needed)."""
        if not self._built:
            self.build()
        return self.root.hash if self.root else ''

    def get_proof(self, leaf_index: int) -> List[Tuple[str, str]]:
        """
        Return Merkle proof for leaf at index.
        Each element is (sibling_hash, direction) where direction is 'left' or 'right'.
        """
        if leaf_index < 0 or leaf_index >= len(self.leaves):
            return []

        if not self._built:
            self.build()

        proof = []
        level = list(self.leaves)
        idx = leaf_index

        while len(level) > 1:
            new_level = []
            for i in range(0, len(level), 2):
                left = level[i]
                right = level[i + 1] if i + 1 < len(level) else level[i]
                combined = (left.hash + right.hash).encode('utf-8')
                parent_hash = hashlib.sha256(combined).hexdigest()
                parent = MerkleNode(hash=parent_hash, left=left, right=right)
                new_level.append(parent)

                if i == idx or i + 1 == idx:
                    if i == idx:
                        proof.append((right.hash, 'right'))
                    else:
                        proof.append((left.hash, 'left'))
                    idx = len(new_level) - 1

            level = new_level

        return proof

    @staticmethod
    def verify_proof(leaf_data: str, proof: List[Tuple[str, str]], root: str) -> bool:
        """Verify a leaf belongs to the tree given its proof and the root hash."""
        current = hashlib.sha256(leaf_data.encode('utf-8')).hexdigest()
        for sibling_hash, direction in proof:
            if direction == 'right':
                combined = (current + sibling_hash).encode('utf-8')
            else:
                combined = (sibling_hash + current).encode('utf-8')
            current = hashlib.sha256(combined).hexdigest()
        return current == root

    def to_manifest(self) -> Dict:
        """Serialize tree to a verifiable manifest."""
        if not self._built:
            self.build()
        return {
            'merkle_root': self.get_root(),
            'leaf_count': len(self.leaves),
            'timestamp': datetime.now(timezone.utc).isoformat(),
            'algorithm': 'SHA-256',
            'leaves': [
                {'index': i, 'hash': node.hash, 'label': node.label}
                for i, node in enumerate(self.leaves)
            ]
        }

    def save_manifest(self, filepath: str = 'merkle_manifest.json'):
        """Write manifest to disk."""
        os.makedirs(os.path.dirname(filepath) or '.', exist_ok=True)
        manifest = self.to_manifest()
        with open(filepath, 'w') as f:
            json.dump(manifest, f, indent=2)
        return manifest

    @classmethod
    def from_manifest(cls, filepath: str) -> 'MerkleTree':
        """Reconstruct leaf hashes from a manifest (for verification)."""
        with open(filepath) as f:
            data = json.load(f)
        tree = cls()
        for leaf in data['leaves']:
            node = MerkleNode(hash=leaf['hash'], label=leaf.get('label', ''))
            tree.leaves.append(node)
        tree.build()
        return tree
