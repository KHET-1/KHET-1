import hashlib
import json
import os
from pathlib import Path
from datetime import datetime, timezone
from typing import List, Dict, Optional
from dataclasses import dataclass, field

from merkle.tree import MerkleTree
from merkle.utils import hash_file


@dataclass
class EvidenceItem:
    filepath: str
    filename: str
    size_bytes: int
    sha256: str
    md5: str
    sealed_at: str
    role: str = 'evidence'


class EvidenceSealer:
    """
    First-touch evidence sealing. Hash on intake. Read-only enforcement.
    Diamond-hand protocol: originals never modified.
    """

    def __init__(self):
        self.items: List[EvidenceItem] = []
        self.merkle = MerkleTree()
        self.sealed = False

    def ingest(self, filepath: str, role: str = 'evidence') -> EvidenceItem:
        """Seal a file on first touch. Returns the evidence item."""
        p = Path(filepath)
        if not p.exists():
            raise FileNotFoundError(f"Evidence file not found: {filepath}")

        data = p.read_bytes()
        item = EvidenceItem(
            filepath=str(p.absolute()),
            filename=p.name,
            size_bytes=len(data),
            sha256=hashlib.sha256(data).hexdigest(),
            md5=hashlib.md5(data).hexdigest(),
            sealed_at=datetime.now(timezone.utc).isoformat(),
            role=role
        )
        self.items.append(item)
        self.merkle.add_leaf(item.sha256, label=f"{role}:{item.filename}")
        return item

    def ingest_directory(self, dirpath: str, extensions=('.pcap', '.pcapng', '.cap')) -> List[EvidenceItem]:
        """Ingest all matching files from a directory."""
        results = []
        for f in sorted(Path(dirpath).iterdir()):
            if f.suffix.lower() in extensions and f.is_file():
                results.append(self.ingest(str(f)))
        return results

    def verify(self, filepath: str) -> bool:
        """Verify a file hasn't been modified since sealing."""
        current_hash = hash_file(filepath)
        for item in self.items:
            if item.filepath == str(Path(filepath).absolute()):
                return current_hash == item.sha256
        return False

    def verify_all(self) -> Dict[str, bool]:
        """Verify all sealed evidence items."""
        results = {}
        for item in self.items:
            try:
                results[item.filename] = self.verify(item.filepath)
            except FileNotFoundError:
                results[item.filename] = False
        return results

    def seal(self) -> str:
        """Finalize the evidence set. Returns Merkle root."""
        self.merkle.build()
        self.sealed = True
        return self.merkle.get_root()

    def to_manifest(self) -> Dict:
        """Export full chain-of-custody manifest."""
        if not self.sealed:
            self.seal()
        return {
            'chain_of_custody': {
                'sealed_at': datetime.now(timezone.utc).isoformat(),
                'item_count': len(self.items),
                'merkle_root': self.merkle.get_root(),
                'network_calls': 0,
                'ai_api_calls': 0,
                'protocol': 'Diamond-hand: read-only, first-touch hash, triple-anchored'
            },
            'items': [
                {
                    'filename': item.filename,
                    'filepath': item.filepath,
                    'size_bytes': item.size_bytes,
                    'sha256': item.sha256,
                    'md5': item.md5,
                    'sealed_at': item.sealed_at,
                    'role': item.role
                }
                for item in self.items
            ],
            'merkle': self.merkle.to_manifest()
        }

    def save_manifest(self, filepath: str = 'chain_of_custody.json'):
        """Write chain of custody to disk."""
        manifest = self.to_manifest()
        Path(filepath).write_text(json.dumps(manifest, indent=2))
        return manifest
