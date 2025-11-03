package com.scar.server.Socket;

import com.scar.server.Service.SessionService;
import io.netty.bootstrap.ServerBootstrap;
import io.netty.channel.*;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.SocketChannel;
import io.netty.channel.socket.nio.NioServerSocketChannel;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class FileTransferServer {
    private static final Logger log = LoggerFactory.getLogger(FileTransferServer.class);
    private static final int PORT = 10000;

    private EventLoopGroup bossGroup;
    private EventLoopGroup workerGroup;
    private final SocketSessionRegistry registry;
    private final SessionService sessionService;

    public FileTransferServer(SocketSessionRegistry registry, SessionService sessionService) {
        this.registry = registry;
        this.sessionService = sessionService;
    }

    @PostConstruct
    public void start() throws Exception {
        bossGroup = new NioEventLoopGroup(1);
        workerGroup = new NioEventLoopGroup(4);

        new Thread(() -> {
            try {
                ServerBootstrap bootstrap = new ServerBootstrap();
                bootstrap.group(bossGroup, workerGroup)
                        .channel(NioServerSocketChannel.class)
                        .childHandler(new ChannelInitializer<SocketChannel>() {
                            @Override
                            protected void initChannel(SocketChannel ch) {
                                ch.pipeline()
                                        // No frame decoder - raw bytes only!
                                        // Handshake: "session_id:role\n" (text)
                                        // binary file data (no framing)
                                        .addLast(new FileTransferHandler(registry, sessionService));
                            }
                        })
                        .option(ChannelOption.SO_BACKLOG, 128)
                        .childOption(ChannelOption.SO_KEEPALIVE, true)
                        .childOption(ChannelOption.TCP_NODELAY, true);

                ChannelFuture future = bootstrap.bind(PORT).sync();
                log.info("Socket Server running on port {}", PORT);
                future.channel().closeFuture().sync();
            } catch (Exception e) {
                log.error("Socket server failed to start", e);
            }
        }, "socket-server-thread").start();
    }

    @PreDestroy
    public void stop() {
        log.info("Shutting down socket server...");
        if (workerGroup != null) {
            workerGroup.shutdownGracefully();
        }
        if (bossGroup != null) {
            bossGroup.shutdownGracefully();
        }
    }
}
