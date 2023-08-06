FROM rust:bookworm


RUN cat nameserver 8.8.8.8 > /etc/resolv.conf

RUN cargo install cargo-make

WORKDIR /downloads

# Get and setup neovim
RUN wget "https://github.com/neovim/neovim/releases/latest/download/nvim-linux64.tar.gz"
RUN tar -xzf nvim-linux64.tar.gz
RUN export PATH=$PATH:./nvim-linux64/bin

# Get and setup zig compiler
RUN wget "https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz"
RUN tar -xzf zig-linux-x86_64-0.11.0.tar.xz
RUN export PATH=$PATH:./zig-linux-x86_64-0.11.0

COPY ./ /test_dir
# Move to dir to do the work
WORKDIR /test_dir


CMD ["cargo", "make", "test"]
