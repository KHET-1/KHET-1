import hashlib
from pathlib import Path


def hash_file(filepath: str) -> str:
    """SHA-256 hash a file."""
    return hashlib.sha256(Path(filepath).read_bytes()).hexdigest()


def hash_string(data: str) -> str:
    """SHA-256 hash a string."""
    return hashlib.sha256(data.encode('utf-8')).hexdigest()


def hash_bytes(data: bytes) -> str:
    """SHA-256 hash raw bytes."""
    return hashlib.sha256(data).hexdigest()


def verify_file(filepath: str, expected_hash: str) -> bool:
    """Verify a file matches its expected hash."""
    return hash_file(filepath) == expected_hash
