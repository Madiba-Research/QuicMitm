use core::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
// use std::ops::Deref;
use std::{fs, sync::Arc};
// use pem::Pem;
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use rustls::server;
use rustls::crypto::ring::sign;
use rcgen::{BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, DnValue::PrintableString,};
use time::{Duration, OffsetDateTime};




fn validity_period() -> (OffsetDateTime, OffsetDateTime) {
	let day = Duration::new(86400, 0);
	let yesterday = OffsetDateTime::now_utc().checked_sub(day).unwrap();
	let tomorrow = OffsetDateTime::now_utc().checked_add(day).unwrap();
	(yesterday, tomorrow)
}


pub struct DynamicCertResolver {
    // resolver: Arc<dyn server::ResolvesServerCert>,
    ca_cert: Certificate,
    ca_key: KeyPair,
}

impl DynamicCertResolver {
    pub fn new(ca_cert_name: &str, ca_key_name: &str) -> Self {

        if !(Path::new(ca_cert_name).exists() && Path::new(ca_key_name).exists()) {
            let (ca_cert_gen, ca_key_gen) = new_ca();

            fs::write(ca_key_name, ca_key_gen.serialize_pem()).unwrap();
            let ca_cert_gen_pem = ca_cert_gen.pem();
            fs::write(ca_cert_name, ca_cert_gen_pem).unwrap();

            panic!("CA certificate and private key have been generated, please import into browser!!!");
        }

        // read ca key pem
        let ca_key_pem = fs::read_to_string(ca_key_name).unwrap();
        let ca_key_pair = KeyPair::from_pem(&ca_key_pem).unwrap();
        
        // read ca cert pem, with from_ca_cert_pem
        let ca_cert_pem = fs::read_to_string(ca_cert_name).unwrap();
        let ca_cert_param = CertificateParams::from_ca_cert_pem(&ca_cert_pem).unwrap();

        println!("is ca? : {:?}", ca_cert_param.is_ca);
        println!("serial number? : {:?}", ca_cert_param.serial_number);

        let ca_der = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(ca_cert_name).unwrap()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .first()
            .unwrap()
            .clone();

        // let ca_pub_key_info = ca_key_pair.public_key_der();

        // generate cert from cert param
        // https://docs.rs/rcgen/latest/rcgen/struct.Certificate.html
        // let my_ca_cert = ca_cert_param.self_signed(&ca_key_pair).unwrap();
        // let my_ca_cert = Certificate::new(ca_cert_param, ca_pub_key_info, ca_der);
        
        let my_ca_cert_new = Certificate::from_der(&ca_der).unwrap();

        // to check the if this cert is same as our ca cert
        // let pem_serialized = my_ca_cert_new.pem();
        // let pem = pem::parse(&pem_serialized).unwrap();
        // fs::write("newcert.pem", pem_serialized.as_bytes()).unwrap();

        DynamicCertResolver {
            // ca_cert: my_ca_cert,
            ca_cert: my_ca_cert_new,
            ca_key: ca_key_pair,
        }
    }
}


impl server::ResolvesServerCert for DynamicCertResolver {
    fn resolve(&self, client_hello: server::ClientHello<'_>) -> Option<Arc<rustls::sign::CertifiedKey>> {
        
        // generate domain cert
        let domain_name = client_hello.server_name()?;
        println!("requesting domain name: {}", domain_name);
        let mut params = CertificateParams::new(vec![domain_name.into()])
            .expect("we know the name is valid");
        let (yesterday, tomorrow) = validity_period();
        params.distinguished_name.push(DnType::CommonName, domain_name);
        params.use_authority_key_identifier_extension = true;
        params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ServerAuth);
        params.not_before = yesterday;
        params.not_after = tomorrow;

        // let key_pair = rcgen::KeyPair::generate().unwrap();
        // let cert = params.self_signed(&key_pair).unwrap();

        // let signer = crypto::ring::sign::any_supported_type(
        //     &rustls::pki_types::PrivateKeyDer::from(
        //         rustls::pki_types::PrivatePkcs8KeyDer::from(key_pair.serialize_der()),
        //     ),
        // )
        let key_pair = KeyPair::generate().unwrap();
        let domain_cert = params.signed_by(&key_pair, &self.ca_cert, &self.ca_key).unwrap();
        let pair_key_der = key_pair.serialize_der();
        let pkcs8_key_der = PrivatePkcs8KeyDer::from(pair_key_der.as_slice());
        let private_key_der = PrivateKeyDer::Pkcs8(pkcs8_key_der);
        let signing_key = sign::any_supported_type(&private_key_der).unwrap();
        // https://docs.rs/rustls/latest/rustls/pki_types/struct.CertificateDer.html
        let cert_der = domain_cert.der().clone();
        // ca cert der
        let ca_cert_der = self.ca_cert.der().clone();
        let cert_der_vec = vec![cert_der, ca_cert_der];

        let certified_key = CertifiedKey::new(cert_der_vec, signing_key);

        Some(Arc::new(certified_key))
    }
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


impl fmt::Debug for DynamicCertResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicCertResolver")
            // skip the Certificate debug as it doesn' implement the Debug trait
            // .field("ca_cert", &self.ca_cert)
            .field("ca_key", &self.ca_key).finish()
    }
}

