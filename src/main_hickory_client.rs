use std::net::Ipv4Addr;
use std::str::FromStr;
use hickory_client::client::{Client, SyncClient};
use hickory_client::udp::UdpClientConnection;
use hickory_client::op::DnsResponse;
use hickory_client::rr::{DNSClass, Name, RData, Record, RecordType};

fn main() {
    let address = "8.8.8.8:53".parse().unwrap();
    let conn = UdpClientConnection::new(address).unwrap();
    let client = SyncClient::new(conn);

    // Specify the name, note the final '.' which specifies it's an FQDN
    let name = Name::from_str("www.google.com.").unwrap();

    // NOTE: see 'Setup a connection' example above
    // Send the query and get a message response, see RecordType for all supported options
    let response: DnsResponse = client.query(&name, DNSClass::IN, RecordType::A).unwrap();

    // Messages are the packets sent between client and server in DNS, DnsResonse's can be
    //  dereferenced to a Message. There are many fields to a Message, It's beyond the scope
    //  of these examples to explain them. See hickory_dns::op::message::Message for more details.
    //  generally we will be interested in the Message::answers
    let answers: &[Record] = response.answers();

    // Records are generic objects which can contain any data.
    //  In order to access it we need to first check what type of record it is
    //  In this case we are interested in A, IPv4 address
    if let Some(RData::A(ref ip)) = answers[0].data() {
        println!("ip is: {}", *ip);
        // assert_eq!(*ip, hickory_client::rr::rdata::A(Ipv4Addr::new(93, 184, 216, 34)))
    } else {
        assert!(false, "unexpected result")
    }
}

