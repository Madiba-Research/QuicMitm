Using existed CA certificate and CA private key
Quic binding finished
quic accepting connection
before conn
2024-11-23T00:50:20.260534Z TRACE first recv:frame{ty=CRYPTO}: quinn_proto::connection: consumed 281 CRYPTO bytes
requesting domain name: acs.m.alibaba.com
2024-11-23T00:50:20.261696Z TRACE first recv: quinn_proto::connection: Handshake keys ready
2024-11-23T00:50:20.261721Z TRACE first recv: quinn_proto::connection: wrote 123 Initial CRYPTO bytes
2024-11-23T00:50:20.261815Z TRACE first recv: quinn_proto::connection: Data keys ready
2024-11-23T00:50:20.261881Z TRACE first recv: quinn_proto::connection: wrote 1239 Handshake CRYPTO bytes
2024-11-23T00:50:20.261980Z TRACE quinn_proto::endpoint: new connection id=0 icid=84b5ed616f814c34197cf889
2024-11-23T00:50:20.262129Z TRACE drive{id=0}:send{space=Initial pn=0}: quinn_proto::connection: ACK ArrayRangeSet([0..1]), Delay = 2108us
2024-11-23T00:50:20.262162Z TRACE drive{id=0}:send{space=Initial pn=0}: quinn_proto::connection: CRYPTO: off 0 len 123
2024-11-23T00:50:20.262298Z TRACE drive{id=0}:send{space=Handshake pn=0}: quinn_proto::connection: CRYPTO: off 0 len 970
2024-11-23T00:50:20.262393Z TRACE drive{id=0}: quinn_proto::connection: sending 1200 bytes in 1 datagrams
2024-11-23T00:50:20.262526Z TRACE drive{id=0}:send{space=Handshake pn=1}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.262619Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.262868Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=1 id=3b3940359ced1ce3
2024-11-23T00:50:20.262901Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=2 id=75a5789bd4196304
2024-11-23T00:50:20.262951Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=3 id=1c5c5b21f61090e3
2024-11-23T00:50:20.262997Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=4 id=61321049817b4fed
2024-11-23T00:50:20.263041Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=5 id=1f534737e5ee63ff
2024-11-23T00:50:20.263087Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=6 id=ed652f3b83022f27
2024-11-23T00:50:20.263148Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=7 id=1cad28c5ed40ceb8
2024-11-23T00:50:20.263270Z TRACE drive{id=0}: quinn_proto::connection: sending 226 bytes in 1 datagrams
2024-11-23T00:50:20.269423Z TRACE drive{id=0}: quinn_proto::connection: got Initial packet (1216 bytes) from 172.30.143.61:38308 using id 66d3355e84a5870b
2024-11-23T00:50:20.269607Z DEBUG drive{id=0}:recv{space=Initial pn=1}:frame{ty=ACK}: quinn_proto::connection: ECN not acknowledged by peer
2024-11-23T00:50:20.270047Z TRACE drive{id=0}: quinn_proto::connection: got Handshake packet (55 bytes) from 172.30.143.61:38308 using id 66d3355e84a5870b
2024-11-23T00:50:20.270120Z TRACE drive{id=0}:recv{space=Handshake pn=0}: quinn_proto::connection: discarding Initial keys
2024-11-23T00:50:20.270208Z TRACE drive{id=0}:recv{space=Handshake pn=0}: quinn_proto::connection: handshake ongoing
2024-11-23T00:50:20.271662Z DEBUG drive{id=0}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
2024-11-23T00:50:20.271695Z DEBUG drive{id=0}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
2024-11-23T00:50:20.276555Z DEBUG drive{id=0}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
2024-11-23T00:50:20.280925Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:20.280986Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=546 count=0 space=Handshake
2024-11-23T00:50:20.281160Z TRACE drive{id=0}:send{space=Handshake pn=2}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.281278Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.281459Z TRACE drive{id=0}:send{space=Handshake pn=3}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.281557Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.318024Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:20.318156Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=1186 count=1 space=Handshake
2024-11-23T00:50:20.318643Z TRACE drive{id=0}:send{space=Handshake pn=4}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.318942Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.319282Z TRACE drive{id=0}:send{space=Handshake pn=5}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.319526Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.390671Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:20.390804Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=1826 count=2 space=Handshake
2024-11-23T00:50:20.391203Z TRACE drive{id=0}:send{space=Handshake pn=6}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.391498Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.391849Z TRACE drive{id=0}:send{space=Handshake pn=7}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.392071Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.534699Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:20.534830Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=2466 count=3 space=Handshake
2024-11-23T00:50:20.535224Z TRACE drive{id=0}:send{space=Handshake pn=8}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.535518Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.535864Z TRACE drive{id=0}:send{space=Handshake pn=9}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.536082Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.819677Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:20.819810Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=3106 count=4 space=Handshake
2024-11-23T00:50:20.820207Z TRACE drive{id=0}:send{space=Handshake pn=10}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.820498Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:20.820848Z TRACE drive{id=0}:send{space=Handshake pn=11}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:20.821072Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:21.389058Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:21.389190Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=3746 count=5 space=Handshake
2024-11-23T00:50:21.389626Z TRACE drive{id=0}:send{space=Handshake pn=12}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:21.389935Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:21.390290Z TRACE drive{id=0}:send{space=Handshake pn=13}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:21.390538Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:22.524197Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:22.524329Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=4386 count=6 space=Handshake
2024-11-23T00:50:22.524766Z TRACE drive{id=0}:send{space=Handshake pn=14}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:22.525033Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:22.525387Z TRACE drive{id=0}:send{space=Handshake pn=15}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:22.525636Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:24.792085Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:24.792215Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=5026 count=7 space=Handshake
2024-11-23T00:50:24.792650Z TRACE drive{id=0}:send{space=Handshake pn=16}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:24.792908Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:24.793257Z TRACE drive{id=0}:send{space=Handshake pn=17}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:24.793504Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:29.327640Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-23T00:50:29.327776Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=5666 count=8 space=Handshake
2024-11-23T00:50:29.328184Z TRACE drive{id=0}:send{space=Handshake pn=18}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:29.328482Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
2024-11-23T00:50:29.328839Z TRACE drive{id=0}:send{space=Handshake pn=19}: quinn_proto::connection: CRYPTO: off 970 len 269
2024-11-23T00:50:29.329071Z TRACE drive{id=0}: quinn_proto::connection: sending 320 bytes in 1 datagrams
