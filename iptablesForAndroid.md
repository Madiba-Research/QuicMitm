forward all tcp and udp packs to 443 into the localhost proxy
remember to set the server's ip address as --to-destination
every time after reboot, remember to set iptables

iptables -t nat -A OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.67
iptables -t nat -A OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.67

to delete a rule for ip table, using flag -D:

iptables -t nat -D OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.67
iptables -t nat -D OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.67

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

delete database:
docker exec -it <container_name> mongosh
show dbs
use <database_name>
db.dropDatabase()
exit



app account:
shaoqi.test@gmail.com
app passwords:
asd123
asd1230-


for http request header, and possible value:
content-encoding:
{"deflate", "union_sdk_encode", "identity", "br", "amz-1.0", "zstd", "msl_v1", "gzip"}
For Nov 6, we only process gzip



while running the proxy, make sure only using the Droid-EV Wifi for internet connection


Play store cannot download without http3


tcpdump -i any -p -s 0 -w /sdcard/capture.pcap


adb pull /sdcard/capture.pcap .


Some problem that cause the failed capture quic/http3 request: quic handshake failed, then the application would reply a close connection with initial, but quinn ignore the initial packet as redundent.


we disable the udp in the second trail, which is the responsiblity of the app to deal with the situation. As in the real world, the access point has the restriction on udp transmission.


Block apex udate, specifically for conscrypto:
    APEX 更新逻辑：
        APEX 更新文件通常下载到 /data/apex/ 目录（如 /data/apex/active 或 /data/apex/tmp），重启后系统会优先加载此目录下的新版本，覆盖 /system/apex 的原始文件。
        单纯锁定 /system/apex 的权限无法阻止 /data/apex 的更新（方法三的漏洞）。

    APEX 文件特性：
        APEX 是只读的压缩镜像，需通过系统服务（apexd）挂载激活，无法直接禁用更新服务（pm disable 对 apexd 无效）。

    原理：阻止系统写入新 APEX 文件到更新目录。
su
# 如果存在子目录（如 /data/apex/active），单独处理：
chattr +i /data/apex/active


// setting up LSPosed scope
// 21 is the mid of cryptologger, the example application is Xiaohongshu
INSERT INTO scope (mid, app_pkg_name, user_id)
VALUES (21, 'com.xingin.xhs', 0)
ON CONFLICT(mid, app_pkg_name, user_id) DO NOTHING;
