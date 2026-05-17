from pathlib import Path


def assert_local_path(path: Path) -> Path:
    resolved = path.expanduser().resolve()
    if str(resolved).startswith(("http://", "https://")):
        raise ValueError("Remote URLs are not allowed in offline clinical runtime")
    return resolved
