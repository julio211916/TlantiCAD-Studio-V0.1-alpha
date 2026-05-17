from functools import lru_cache
from pathlib import Path

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="TLANTI_", env_file=".env", extra="ignore")

    offline_only: bool = True
    enable_docs: bool = True
    database_url: str = "sqlite:///./.tlanticad/local-backend/tlanticad.db"
    redis_url: str = "redis://127.0.0.1:6379/0"
    model_registry_path: Path = Field(default=Path("backend/ml/configs/model_registry.json"))
    artifact_root: Path = Field(default=Path(".tlanticad/local-backend/artifacts"))
    max_dicom_instances_browser: int = 512
    max_browser_import_bytes: int = 512 * 1024 * 1024
    cors_origins: list[str] = Field(default_factory=lambda: [
        "http://127.0.0.1:1420",
        "http://localhost:1420",
        "tauri://localhost",
        "http://tauri.localhost",
        "https://tauri.localhost",
    ])


@lru_cache
def get_settings() -> Settings:
    return Settings()
