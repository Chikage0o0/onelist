# 设置多阶段构建基础镜像
FROM alpine:latest as builder

# # 安装必要的工具和依赖
RUN apk add --no-cache build-base curl yarn && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 

# 设置工作目录
WORKDIR /build


# 复制源码
COPY . .

RUN cd ui && yarn && yarn build && cd ..

RUN source $HOME/.cargo/env && cargo build --release

# 使用Alpine镜像作为最终运行时环境
FROM alpine:latest
WORKDIR /app


COPY --from=builder /build/target/release/onelist /app/onelist

RUN chmod +x /app/onelist

ENTRYPOINT [ "/app/onelist" ]
