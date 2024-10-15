forward all tcp and udp packs to 443 into the localhost proxy
remember to set the server's ip address as --to-destination
every time after reboot, remember to set iptables

iptables -t nat -A OUTPUT -p tcp --dport 443 -j DNAT --to-destination 172.30.143.77
iptables -t nat -A OUTPUT -p udp --dport 443 -j DNAT --to-destination 172.30.143.77

add cacert to the phone:
https://github.com/Magisk-Modules-Alt-Repo/custom-certificate-authorities?tab=readme-ov-file

