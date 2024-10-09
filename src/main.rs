use std::{fs::{self, File}, io::BufReader};

use rcgen::{
	BasicConstraints, Certificate, CertificateParams, DnType, DnValue::PrintableString,
	ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use time::{Duration, OffsetDateTime};

/// Example demonstrating signing end-entity certificate with ca
fn main() {
	let (ca, ca_key) = new_ca();
	let end_entity = new_end_entity(&ca, &ca_key);

	let end_entity_pem = end_entity.pem();
	// println!("directly signed end-entity certificate: {end_entity_pem}");
    // let pem_end = pem::parse(&end_entity_pem).unwrap();
    fs::write("demoendcert.pem", end_entity_pem);

	let ca_cert_pem = ca.pem();
	// println!("ca certificate: {ca_cert_pem}");
    // let pem_ca = pem::parse(&ca_cert_pem).unwrap();
    fs::write("democacert.pem", ca_cert_pem);

    let ca_cert_pem = fs::read_to_string("democacert.pem").unwrap();
    let ca_cert_param = CertificateParams::from_ca_cert_pem(&ca_cert_pem).unwrap();
    println!("is ca? : {:?}", ca_cert_param.is_ca);
    println!("serial number? : {:?}", ca_cert_param.serial_number);

    let ca_der = rustls_pemfile::certs(&mut BufReader::new(&mut File::open("democacert.pem").unwrap()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    let my_ca_cert_new = Certificate::from_der(&ca_der).unwrap();

    let pem_serialized = my_ca_cert_new.pem();
    // let pem = pem::parse(&pem_serialized).unwrap();
    fs::write("newdemocacert.pem", pem_serialized.as_bytes()).unwrap();

}

fn new_ca() -> (Certificate, KeyPair) {
	let mut params =
		CertificateParams::new(Vec::default()).expect("empty subject alt name can't produce error");
	let (yesterday, tomorrow) = validity_period();
	params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
	params.distinguished_name.push(
		DnType::CountryName,
		PrintableString("BR".try_into().unwrap()),
	);
	params
		.distinguished_name
		.push(DnType::OrganizationName, "Crab widgits SE");
	params.key_usages.push(KeyUsagePurpose::DigitalSignature);
	params.key_usages.push(KeyUsagePurpose::KeyCertSign);
	params.key_usages.push(KeyUsagePurpose::CrlSign);

	params.not_before = yesterday;
	params.not_after = tomorrow;

	let key_pair = KeyPair::generate().unwrap();

	(params.self_signed(&key_pair).unwrap(), key_pair)
}

fn new_end_entity(ca: &Certificate, ca_key: &KeyPair) -> Certificate {
	let name = "entity.other.host";
	let mut params = CertificateParams::new(vec![name.into()]).expect("we know the name is valid");
	let (yesterday, tomorrow) = validity_period();
	params.distinguished_name.push(DnType::CommonName, name);
	params.use_authority_key_identifier_extension = true;
	params.key_usages.push(KeyUsagePurpose::DigitalSignature);
	params
		.extended_key_usages
		.push(ExtendedKeyUsagePurpose::ServerAuth);
	params.not_before = yesterday;
	params.not_after = tomorrow;

	let key_pair = KeyPair::generate().unwrap();
	params.signed_by(&key_pair, ca, ca_key).unwrap()
}

fn validity_period() -> (OffsetDateTime, OffsetDateTime) {
	let day = Duration::new(86400, 0);
	let yesterday = OffsetDateTime::now_utc().checked_sub(day).unwrap();
	let tomorrow = OffsetDateTime::now_utc().checked_add(day).unwrap();
	(yesterday, tomorrow)
}