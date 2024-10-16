forward all tcp and udp packs to 443 into the localhost proxy
remember to set the server's ip address as --to-destination
every time after reboot, remember to set iptables

iptables -t nat -A OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.77
iptables -t nat -A OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.77

add cacert to the phone:
https://github.com/Magisk-Modules-Alt-Repo/custom-certificate-authorities?tab=readme-ov-file


A problem for http2 proxy to server:
Connection failed: hyper::Error(Http2, Error { kind: GoAway(b"", FRAME_SIZE_ERROR, Library) })
https://users.rust-lang.org/t/http2-0-frame-size-error-what-doest-that-mean/81106

Solved:
because http1 is default for alpn. Thus, to build http2 connection between proxy and server,
One should explicitly set tls config alpn as http2.


Todo:
For now most h3 request can go through proxy but cannot be 100% processed successfully. 

