//! Tipos de erro unificados para Rusdi webOS.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RusdiError {
    #[error("Falha ao inicializar EGL: {0}")]
    EglInit(String),

    #[error("Falha ao criar superfície EGL: {0}")]
    EglSurface(String),

    #[error("Erro no SDL2: {0}")]
    Sdl2(#[from] sdl2::Error),

    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de serialização JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Erro de configuração: {0}")]
    Config(String),

    #[error("Falha no player de mídia: {0}")]
    MediaPlayer(String),

    #[error("Erro no banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Erro no Python bridge: {0}")]
    Python(String),

    #[error("Erro de rede: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Erro de sinal: {0}")]
    Signal(#[from] signal_hook::Error),

    #[error("Recurso não encontrado: {0}")]
    NotFound(String),

    #[error("Operação não suportada: {0}")]
    Unsupported(String),

    #[error("Falha no FFmpeg: {0}")]
    Ffmpeg(#[from] ffmpeg_next::Error),

    #[error("Erro JSON-RPC: {0}")]
    JsonRpc(String),
}

pub type Result<T> = std::result::Result<T, RusdiError>;
