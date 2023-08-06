FROM rust:bookworm

RUN cargo install cargo-make

WORKDIR /downloads

# Get and setup neovim
RUN wget "https://github.com/neovim/neovim/releases/latest/download/nvim-linux64.tar.gz"
RUN tar -xzf nvim-linux64.tar.gz

# Get and setup zig compiler
RUN wget "https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz"
RUN tar -xf zig-linux-x86_64-0.11.0.tar.xz

COPY ./ /test_dir
# Move to dir to do the work
WORKDIR /test_dir


CMD ["cargo", "make", "docker_test"]
