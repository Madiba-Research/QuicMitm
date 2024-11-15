forward all tcp and udp packs to 443 into the localhost proxy
remember to set the server's ip address as --to-destination
every time after reboot, remember to set iptables

iptables -t nat -A OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.95
iptables -t nat -A OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.95

to delete a rule for ip table, using flag -D:

iptables -t nat -D OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.95
iptables -t nat -D OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.95

list rules: iptables -t nat -L OUTPUT --line-numbers


Magisk repo:
https://github.com/Magisk-Modules-Alt-Repo
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
Solved:
always use bi_stream to receive requests from client.


to run proxy:
cargo run --bin main_h1_h2_h3


run mongodb in docker:
sudo docker run --name mongodb -p 27017:27017 -d mongodb/mongodb-community-server:latest
sudo docker ps
sudo docker ps -a
sudo docker start mongodb
sudo docker stop mongodb
request data location: database("requestdb").collection("httpreq")


app account:
shaoqi.test@gmail.com
app passwords:
asd123
asd1230-


for http request header, and possible value:
content-encoding:
{"deflate", "union_sdk_encode", "identity", "br", "amz-1.0", "zstd", "msl_v1", "gzip"}
For Nov 6, we only process gzip


problem with proxy
Request { method: POST, uri: https://quake-pa.googleapis.com:443/google.internal.android.location.quake.v1.QuakeApiService/NodeOnline, version: HTTP/2.0, headers: {"user-agent": "grpc-java-okhttp/1.68.0-SNAPSHOT", "content-type": "application/grpc", "te": "trailers", "x-android-package": "com.google.android.gms", "x-android-cert": "38918A453D07199354F8B19AF05EC6562CED5788", "x-goog-skey": "4cc91a4", "x-goog-spatula":




