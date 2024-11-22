2024-11-21T22:03:13.950649Z TRACE first recv:frame{ty=CRYPTO}: quinn_proto::connection: consumed 513 CRYPTO bytes
2024-11-21T22:03:13.950841Z TRACE first recv:frame{ty=CRYPTO}: quinn_proto::connection: consumed 79 CRYPTO bytes
requesting domain name: app-measurement.com
2024-11-21T22:03:13.953279Z TRACE first recv: quinn_proto::connection: Handshake keys ready
2024-11-21T22:03:13.953318Z TRACE first recv: quinn_proto::connection: wrote 90 Initial CRYPTO bytes
2024-11-21T22:03:13.953408Z TRACE first recv: quinn_proto::connection: Data keys ready
2024-11-21T22:03:13.953467Z TRACE first recv: quinn_proto::connection: wrote 1237 Handshake CRYPTO bytes
2024-11-21T22:03:13.953515Z TRACE quinn_proto::endpoint: new connection id=4 icid=96b9ef402cabeea8
2024-11-21T22:03:13.953597Z TRACE drive{id=4}:send{space=Initial pn=0}: quinn_proto::connection: ACK ArrayRangeSet([1..2]), Delay = 5289us
2024-11-21T22:03:13.953620Z TRACE drive{id=4}:send{space=Initial pn=0}: quinn_proto::connection: CRYPTO: off 0 len 90
2024-11-21T22:03:13.953686Z TRACE drive{id=4}:send{space=Handshake pn=0}: quinn_proto::connection: CRYPTO: off 0 len 1027
2024-11-21T22:03:13.953728Z TRACE drive{id=4}: quinn_proto::connection: sending 1200 bytes in 1 datagrams
2024-11-21T22:03:13.953825Z TRACE drive{id=4}:send{space=Handshake pn=1}: quinn_proto::connection: CRYPTO: off 1027 len 210
2024-11-21T22:03:13.953895Z TRACE drive{id=4}: quinn_proto::connection: sending 249 bytes in 1 datagrams
2024-11-21T22:03:13.954104Z TRACE drive{id=4}:send{space=Data pn=0}: quinn_proto::connection: NEW_CONNECTION_ID sequence=1 id=6e845e193cb7def9
2024-11-21T22:03:13.954215Z TRACE drive{id=4}: quinn_proto::connection: sending 46 bytes in 1 datagrams
2024-11-21T22:03:13.983655Z TRACE drive{id=4}: quinn_proto::connection: got Initial packet (1210 bytes) from 172.30.143.61:54222 using id 620db72d4758b6e2
2024-11-21T22:03:13.983828Z DEBUG drive{id=4}:recv{space=Initial pn=2}:frame{ty=ACK}: quinn_proto::connection: ECN not acknowledged by peer
2024-11-21T22:03:13.984345Z TRACE drive{id=4}: quinn_proto::connection: got Handshake packet (40 bytes) from 172.30.143.61:54222 using id 620db72d4758b6e2
2024-11-21T22:03:13.984422Z TRACE drive{id=4}:recv{space=Handshake pn=3}: quinn_proto::connection: discarding Initial keys
2024-11-21T22:03:13.984507Z TRACE drive{id=4}:recv{space=Handshake pn=3}: quinn_proto::connection: handshake ongoing
2024-11-21T22:03:13.984553Z TRACE drive{id=4}: quinn_proto::connection: got Handshake packet (40 bytes) from 172.30.143.61:54222 using id 620db72d4758b6e2
2024-11-21T22:03:13.984719Z TRACE drive{id=4}:recv{space=Handshake pn=4}: quinn_proto::connection: handshake ongoing
2024-11-21T22:03:14.000048Z TRACE drive{id=4}: quinn_proto::connection: got Handshake packet (73 bytes) from 172.30.143.61:54222 using id 620db72d4758b6e2
2024-11-21T22:03:14.000370Z TRACE drive{id=4}:recv{space=Handshake pn=5}:frame{ty=CRYPTO}: quinn_proto::connection: consumed 36 CRYPTO bytes
2024-11-21T22:03:14.001315Z TRACE drive{id=4}:recv{space=Handshake pn=5}: quinn_proto::connection: wrote 324 Data CRYPTO bytes
2024-11-21T22:03:14.001430Z TRACE drive{id=4}:recv{space=Handshake pn=5}: quinn_proto::connection: discarding Handshake keys
2024-11-21T22:03:14.001533Z TRACE drive{id=4}:recv{space=Handshake pn=5}: quinn_proto::connection: established
