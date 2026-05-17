from __future__ import annotations

from celery import Celery

from backend.app.core.settings import get_settings

settings = get_settings()
celery_app = Celery(
    "tlanticad_local_backend",
    broker=settings.redis_url,
    backend=settings.redis_url,
)
celery_app.conf.update(
    task_track_started=True,
    worker_prefetch_multiplier=1,
    task_acks_late=True,
)

