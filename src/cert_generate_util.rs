use std::{fmt, fs};
use std::collections::HashMap;
use std::sync::Arc;
use rcgen::{Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, KeyPair, KeyUsagePurpose};

use rustls::pki_types::{PrivateKeyDer, PrivatePkcs1KeyDer};
use rustls::sign::CertifiedKey;
use rustls::server::ResolvesServerCert;
use rustls::crypto::ring::sign;
use time::{Duration, OffsetDateTime};


fn validity_period() -> (OffsetDateTime, OffsetDateTime) {
	let day = Duration::new(86400, 0);
	let yesterday = OffsetDateTime::now_utc().checked_sub(day).unwrap();
	let tomorrow = OffsetDateTime::now_utc().checked_add(day).unwrap();
	(yesterday, tomorrow)
}



struct DynamicCertResolver {
    cert_cache: HashMap<String, Arc<CertifiedKey>>,
    ca_cert: Certificate,
    ca_key: KeyPair,
}


impl DynamicCertResolver {
    fn new(ca_cert_name: &str, ca_key_name: &str) -> Self {

        // read ca key pem
        let ca_key_pem = fs::read_to_string(ca_key_name).unwrap();
        let ca_key_pair = KeyPair::from_pem(&ca_key_pem).unwrap();
        
        // read ca cert pem, with from_ca_cert_pem
        let ca_cert_pem = fs::read_to_string(ca_cert_name).unwrap();
        let ca_cert_param = CertificateParams::from_ca_cert_pem(&ca_cert_pem).unwrap();
        
        // generate cert from cert param
        let my_ca_cert = ca_cert_param.self_signed(&ca_key_pair).unwrap();

        DynamicCertResolver {
            cert_cache: HashMap::<String, Arc<CertifiedKey>>::new(),
            ca_cert: my_ca_cert,
            ca_key: ca_key_pair,
        }
    }


    // the function only update the cert_cache, to get the certifiedkey:
    // generate_key_for_domain → cert_cache.get();
    fn generate_cert_key_for_domain (&self, domain_name: &str) {
        if let Some(cert_key) = self.cert_cache.get(domain_name) {
            return;
        }
        // for end entity
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

        let alg = &rcgen::PKCS_RSA_SHA256;
        let key_pair = KeyPair::generate_for(alg).unwrap();
        let domain_cert = params.signed_by(&key_pair, &self.ca_cert, &self.ca_key).unwrap();
        
        // let certified_key = CertifiedKey { cert: domain_cert, key_pair: key_pair };
        // convert to rustls certifiedkey
        // https://docs.rs/rustls/latest/rustls/sign/trait.SigningKey.html
        
        todo!("serialize_der only for pkcs8");
        let pair_key_der = key_pair.serialize_der();
        let pkcs1_key_der = PrivatePkcs1KeyDer::from(pair_key_der.as_slice());
        let private_key_der = PrivateKeyDer::Pkcs1(pkcs1_key_der);
        let signing_key = sign::any_supported_type(&private_key_der).unwrap();
        // https://docs.rs/rustls/latest/rustls/pki_types/struct.CertificateDer.html
        let cert_der = *domain_cert.der();
        let cert_der_vec = vec![cert_der];

        let certified_key = CertifiedKey::new(cert_der_vec, signing_key);
        self.cert_cache.insert(String::from(domain_name), Arc::new(certified_key));
        
    }
}


impl ResolvesServerCert for DynamicCertResolver {
    fn resolve(&self, client_hello: rustls::server::ClientHello<'_>) -> Option<std::sync::Arc<rustls::sign::CertifiedKey>> {
        if let Some(sni) = client_hello.server_name() {
            self.generate_cert_key_for_domain(sni);
            let gen_cert = self.cert_cache.get(sni);
            return gen_cert.cloned();
        }
        None
    }
}

impl fmt::Debug for DynamicCertResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicCertResolver")
            // skip the Certificate debug as it doesn' implement the Debug trait
            // .field("cert_cache", &self.cert_cache)
            // .field("ca_cert", &self.ca_cert)
            .field("ca_key", &self.ca_key).finish()
    }
}


