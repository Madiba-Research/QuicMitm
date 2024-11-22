2024-11-21T22:02:58.804495Z TRACE first recv:frame{ty=CRYPTO}: quinn_proto::connection: consumed 281 CRYPTO bytes
requesting domain name: acs.m.alibaba.com
2024-11-21T22:02:58.807112Z TRACE first recv: quinn_proto::connection: Handshake keys ready
2024-11-21T22:02:58.807146Z TRACE first recv: quinn_proto::connection: wrote 123 Initial CRYPTO bytes
2024-11-21T22:02:58.807294Z TRACE first recv: quinn_proto::connection: Data keys ready
2024-11-21T22:02:58.807486Z TRACE first recv: quinn_proto::connection: wrote 1238 Handshake CRYPTO bytes
2024-11-21T22:02:58.807629Z TRACE quinn_proto::endpoint: new connection id=0 icid=e6b86439860eec3c01c066a6
2024-11-21T22:02:58.807789Z TRACE drive{id=0}:send{space=Initial pn=0}: quinn_proto::connection: ACK ArrayRangeSet([0..1]), Delay = 5114us
2024-11-21T22:02:58.807828Z TRACE drive{id=0}:send{space=Initial pn=0}: quinn_proto::connection: CRYPTO: off 0 len 123
2024-11-21T22:02:58.807915Z TRACE drive{id=0}:send{space=Handshake pn=0}: quinn_proto::connection: CRYPTO: off 0 len 970
2024-11-21T22:02:58.808004Z TRACE drive{id=0}:send{space=Handshake pn=1}: quinn_proto::connection: CRYPTO: off 970 len 268
2024-11-21T22:02:58.808059Z TRACE drive{id=0}: quinn_proto::connection: sending 1519 bytes in 2 datagrams
2024-11-21T22:02:58.808140Z ERROR drive{id=0}: quinn_udp::imp: got transmit error, halting segmentation offload
2024-11-21T22:02:58.808187Z  WARN drive{id=0}: quinn_udp: sendmsg error: Os { code: 5, kind: Uncategorized, message: "Input/output error" }, Transmit: { destination: 172.30.143.61:32780, src_ip: Some(172.30.143.58), ecn: Some(Ect0), len: 1519, segment_size: Some(1200) }
2024-11-21T22:02:58.808458Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=1 id=a85a7d162e90ed63
2024-11-21T22:02:58.808502Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=2 id=1b656fda919c5b34
2024-11-21T22:02:58.808544Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=3 id=5702cf10a0112ec7
2024-11-21T22:02:58.808601Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=4 id=5a2a4c457f402a34
2024-11-21T22:02:58.808643Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=5 id=616c92e60459ea4a
2024-11-21T22:02:58.808721Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=6 id=ca32d832aad83cb9
2024-11-21T22:02:58.808754Z TRACE drive{id=0}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=7 id=469f6b50fb344406
2024-11-21T22:02:58.808810Z TRACE drive{id=0}: quinn_proto::connection: sending 226 bytes in 1 datagrams
2024-11-21T22:02:59.531934Z TRACE drive{id=0}: quinn_proto::connection: got Initial packet (1216 bytes) from 172.30.143.61:32780 using id e6b86439860eec3c01c066a6
2024-11-21T22:02:59.533373Z TRACE drive{id=0}: quinn_proto::connection: got Initial packet (1216 bytes) from 172.30.143.61:32780 using id e6b86439860eec3c01c066a6
2024-11-21T22:02:59.534677Z TRACE drive{id=0}:send{space=Initial pn=1}: quinn_proto::connection: ACK ArrayRangeSet([0..3]), Delay = 3273us
2024-11-21T22:02:59.534926Z TRACE drive{id=0}: quinn_proto::connection: sending 53 bytes in 1 datagrams
2024-11-21T22:02:59.808300Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-21T22:02:59.808439Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=1745 count=0 space=Initial
2024-11-21T22:02:59.808986Z TRACE drive{id=0}:send{space=Initial pn=2}: quinn_proto::connection: CRYPTO: off 0 len 123
2024-11-21T22:02:59.809186Z TRACE drive{id=0}:send{space=Initial pn=2}: quinn_proto::connection::packet_builder: PADDING * 1026
2024-11-21T22:02:59.809535Z TRACE drive{id=0}: quinn_proto::connection: sending 1200 bytes in 1 datagrams
2024-11-21T22:02:59.810000Z TRACE drive{id=0}:send{space=Initial pn=3}: quinn_proto::connection: CRYPTO: off 0 len 123
2024-11-21T22:02:59.810174Z TRACE drive{id=0}:send{space=Initial pn=3}: quinn_proto::connection::packet_builder: PADDING * 1026
2024-11-21T22:02:59.810505Z TRACE drive{id=0}: quinn_proto::connection: sending 1200 bytes in 1 datagrams
2024-11-21T22:02:59.822096Z TRACE drive{id=0}: quinn_proto::connection: got Initial packet (1216 bytes) from 172.30.143.61:32780 using id d782e4905e29a0c5
2024-11-21T22:02:59.822546Z TRACE drive{id=0}:recv{space=Initial pn=3}:frame{ty=ACK}: quinn_proto::connection: packets lost: [0], bytes lost: 180
2024-11-21T22:02:59.822725Z DEBUG drive{id=0}:recv{space=Initial pn=3}:frame{ty=ACK}: quinn_proto::connection: ECN not acknowledged by peer
2024-11-21T22:02:59.824113Z TRACE drive{id=0}: quinn_proto::connection: got Initial packet (1216 bytes) from 172.30.143.61:32780 using id d782e4905e29a0c5
2024-11-21T22:02:59.825734Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-21T22:02:59.825842Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=1565 count=0 space=Handshake
2024-11-21T22:02:59.826190Z TRACE drive{id=0}:send{space=Handshake pn=2}: quinn_proto::connection: CRYPTO: off 0 len 970
2024-11-21T22:02:59.826478Z TRACE drive{id=0}: quinn_proto::connection: sending 1020 bytes in 1 datagrams
2024-11-21T22:02:59.826660Z TRACE drive{id=0}:send{space=Handshake pn=3}: quinn_proto::connection: CRYPTO: off 970 len 268
2024-11-21T22:02:59.826722Z TRACE drive{id=0}: quinn_proto::connection: sending 319 bytes in 1 datagrams
2024-11-21T22:02:59.835543Z TRACE drive{id=0}: quinn_proto::connection: got Handshake packet (54 bytes) from 172.30.143.61:32780 using id d782e4905e29a0c5
2024-11-21T22:02:59.835611Z TRACE drive{id=0}:recv{space=Handshake pn=0}: quinn_proto::connection: discarding Initial keys
2024-11-21T22:02:59.835698Z TRACE drive{id=0}:recv{space=Handshake pn=0}:frame{ty=ACK}: quinn_proto::connection: packets lost: [0, 1], bytes lost: 1339
2024-11-21T22:02:59.835776Z TRACE drive{id=0}:recv{space=Handshake pn=0}: quinn_proto::connection: handshake ongoing
2024-11-21T22:02:59.846108Z DEBUG drive{id=0}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
2024-11-21T22:02:59.846134Z DEBUG drive{id=0}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
quic accepting connection
before conn
2024-11-21T22:02:59.848473Z TRACE first recv:frame{ty=CRYPTO}: quinn_proto::connection: consumed 281 CRYPTO bytes
requesting domain name: acs.m.alibaba.com
2024-11-21T22:02:59.849309Z TRACE first recv: quinn_proto::connection: Handshake keys ready
2024-11-21T22:02:59.849327Z TRACE first recv: quinn_proto::connection: wrote 123 Initial CRYPTO bytes
2024-11-21T22:02:59.849400Z TRACE first recv: quinn_proto::connection: Data keys ready
2024-11-21T22:02:59.849517Z TRACE first recv: quinn_proto::connection: wrote 1234 Handshake CRYPTO bytes
2024-11-21T22:02:59.849594Z TRACE quinn_proto::endpoint: new connection id=1 icid=8829c7fb5c4d8ca8a12e2de1
2024-11-21T22:02:59.849698Z TRACE drive{id=1}:send{space=Initial pn=0}: quinn_proto::connection: ACK ArrayRangeSet([0..1]), Delay = 1519us
2024-11-21T22:02:59.849722Z TRACE drive{id=1}:send{space=Initial pn=0}: quinn_proto::connection: CRYPTO: off 0 len 123
2024-11-21T22:02:59.849842Z TRACE drive{id=1}:send{space=Handshake pn=0}: quinn_proto::connection: CRYPTO: off 0 len 970
2024-11-21T22:02:59.849945Z TRACE drive{id=1}: quinn_proto::connection: sending 1200 bytes in 1 datagrams
2024-11-21T22:02:59.850063Z TRACE drive{id=1}:send{space=Handshake pn=1}: quinn_proto::connection: CRYPTO: off 970 len 264
2024-11-21T22:02:59.850149Z TRACE drive{id=1}: quinn_proto::connection: sending 315 bytes in 1 datagrams
2024-11-21T22:02:59.850403Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=1 id=9fb6e1e45fcc4fe8
2024-11-21T22:02:59.850450Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=2 id=6a8344de9b7330e7
2024-11-21T22:02:59.850518Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=3 id=8eb8e7025ccb2741
2024-11-21T22:02:59.850560Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=4 id=638a8b214c83ca80
2024-11-21T22:02:59.850603Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=5 id=c309b70106f2b065
2024-11-21T22:02:59.850648Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=6 id=975945d60891ad62
2024-11-21T22:02:59.850697Z TRACE drive{id=1}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=7 id=c2c269dcc6ff10a1
2024-11-21T22:02:59.850777Z TRACE drive{id=1}: quinn_proto::connection: sending 226 bytes in 1 datagrams
2024-11-21T22:02:59.856021Z TRACE drive{id=1}: quinn_proto::connection: got Initial packet (1216 bytes) from 172.30.143.61:41165 using id 69ae843a747d533b
2024-11-21T22:02:59.856143Z DEBUG drive{id=1}:recv{space=Initial pn=1}:frame{ty=ACK}: quinn_proto::connection: ECN not acknowledged by peer
2024-11-21T22:02:59.856570Z TRACE drive{id=1}: quinn_proto::connection: got Handshake packet (55 bytes) from 172.30.143.61:41165 using id 69ae843a747d533b
2024-11-21T22:02:59.856620Z TRACE drive{id=1}:recv{space=Handshake pn=0}: quinn_proto::connection: discarding Initial keys
2024-11-21T22:02:59.856701Z TRACE drive{id=1}:recv{space=Handshake pn=0}: quinn_proto::connection: handshake ongoing
2024-11-21T22:02:59.860056Z TRACE drive{id=0}: quinn_proto::connection: timeout timer=LossDetection
2024-11-21T22:02:59.860078Z TRACE drive{id=0}: quinn_proto::connection: PTO fired in_flight=545 count=0 space=Handshake
2024-11-21T22:02:59.860115Z DEBUG drive{id=1}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
2024-11-21T22:02:59.860165Z DEBUG drive{id=1}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
2024-11-21T22:02:59.860249Z TRACE drive{id=0}:send{space=Handshake pn=4}: quinn_proto::connection: CRYPTO: off 970 len 268
2024-11-21T22:02:59.860357Z TRACE drive{id=0}: quinn_proto::connection: sending 319 bytes in 1 datagrams
2024-11-21T22:02:59.860440Z TRACE drive{id=0}:send{space=Handshake pn=5}: quinn_proto::connection: CRYPTO: off 970 len 268
2024-11-21T22:02:59.860504Z TRACE drive{id=0}: quinn_proto::connection: sending 319 bytes in 1 datagrams
2024-11-21T22:02:59.866989Z TRACE drive{id=1}: quinn_proto::connection: timeout timer=LossDetection
2024-11-21T22:02:59.867026Z TRACE drive{id=1}: quinn_proto::connection: PTO fired in_flight=541 count=0 space=Handshake
2024-11-21T22:02:59.867139Z TRACE drive{id=1}:send{space=Handshake pn=2}: quinn_proto::connection: CRYPTO: off 970 len 264
2024-11-21T22:02:59.867230Z TRACE drive{id=1}: quinn_proto::connection: sending 315 bytes in 1 datagrams
2024-11-21T22:02:59.867303Z TRACE drive{id=1}:send{space=Handshake pn=3}: quinn_proto::connection: CRYPTO: off 970 len 264
2024-11-21T22:02:59.867355Z TRACE drive{id=1}: quinn_proto::connection: sending 315 bytes in 1 datagrams
quic accepting connection
before conn
2024-11-21T22:02:59.870150Z DEBUG drive{id=0}: quinn_proto::connection::packet_crypto: discarding unexpected Initial packet (1216 bytes)
