
const ROOT_NAME_SERVERS: [&str; 13] = [
    // See: root-servers.org
    "198.41.0.4",
    "199.9.14.201",
    "192.33.4.12",
    "199.7.91.13",
    "192.203.230.10",
    "192.5.5.241",
    "192.112.36.4",
    "198.97.190.53",
    "192.36.148.17",
    "192.58.128.30",
    "193.0.14.129",
    "199.7.83.42",
    "202.12.27.33",
];

pub fn resolve(fqdn: &str, qtype: u16) {
    println!("{:?} の type {:?} を解決していくよ！", fqdn, qtype);
}