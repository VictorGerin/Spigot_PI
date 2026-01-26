# Etapa 1: Builder (Ubuntu + Rust + OpenMPI)
FROM ubuntu:22.04 AS builder

# Evita prompts interativos durante a instalação
ENV DEBIAN_FRONTEND=noninteractive

# Instala dependências de compilação
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libopenmpi-dev \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Instala o Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Prepara o diretório de trabalho
WORKDIR /app

# OTIMIZAÇÃO DE CACHE:
# 1. Copia apenas os manifestos do Rust
COPY Cargo.toml Cargo.lock ./

# 2. Cria estrutura dummy para enganar o compilador
RUN mkdir -p src/bin && \
    echo "fn main() {println!(\"dummy\")}" > src/main.rs && \
    echo "fn main() {println!(\"dummy\")}" > src/bin/spigot_mpi.rs

# 3. Compila apenas as dependências (isso cria uma camada de cache)
RUN cargo build --release --features mpi

# 4. Remove os arquivos dummy e copia o código real
RUN rm -f target/release/deps/spigot_pi* target/release/deps/spigot_mpi*
COPY . .

# 5. Compila o projeto real (vai usar as dependências cacheadas do passo 3)
RUN cargo build --release --features mpi

# Etapa 2: Runner (Alpine + OpenMPI + Compatibilidade Glibc)
FROM alpine:latest

# Instala dependências de runtime
RUN apk add --no-cache \
    openmpi \
    gcompat \
    libgcc \
    openssh \
    openssh-client \
    && ssh-keygen -A \
    && echo "PermitRootLogin yes" >> /etc/ssh/sshd_config \
    && echo "PasswordAuthentication no" >> /etc/ssh/sshd_config \
    && echo "StrictModes no" >> /etc/ssh/sshd_config \
    && rm -rf /var/cache/apk/*

WORKDIR /app

# Copia APENAS o binário compilado do estágio anterior
COPY --from=builder /app/target/release/spigot_mpi .

# Garante permissão de execução
RUN chmod +x spigot_mpi

# Comando padrão
CMD ["./spigot_mpi"]
