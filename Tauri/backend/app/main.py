from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from backend.app.api.v1 import health, inference, models, studies, tasks, tool_settings
from backend.app.api.ws.events import router as ws_router
from backend.app.core.settings import get_settings


def create_app() -> FastAPI:
    settings = get_settings()
    app = FastAPI(
        title="TlantiCAD Local Dental Backend",
        version="0.1.0",
        openapi_version="3.1.0",
        docs_url="/docs" if settings.enable_docs else None,
        redoc_url=None,
    )
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.cors_origins,
        allow_credentials=False,
        allow_methods=["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"],
        allow_headers=["Authorization", "Content-Type", "X-TlantiCAD-Request"],
        max_age=600,
    )
    app.include_router(health.router, prefix="/api/v1", tags=["health"])
    app.include_router(studies.router, prefix="/api/v1/studies", tags=["studies"])
    app.include_router(inference.router, prefix="/api/v1/inference", tags=["inference"])
    app.include_router(tasks.router, prefix="/api/v1/tasks", tags=["tasks"])
    app.include_router(models.router, prefix="/api/v1/models", tags=["models"])
    app.include_router(tool_settings.router, prefix="/api/v1/tool-settings", tags=["tool-settings"])
    app.include_router(ws_router, prefix="/api/ws", tags=["ws"])
    return app


app = create_app()
