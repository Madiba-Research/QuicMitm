forward all tcp and udp packs to 443 into the localhost proxy
remember to set the server's ip address as --to-destination

iptables -t nat -A OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.77
iptables -t nat -A OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.77

add cacert to the phone:
https://infosecwriteups.com/adding-root-certificate-to-android-with-magisk-module-92493a7e9e4f
https://github.com/NVISOsecurity/MagiskTrustUserCerts?tab=readme-ov-file

