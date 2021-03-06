use core::time;
use rand::Rng;
use std::net::UdpSocket;
use std::thread::sleep;

use crate::message;

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

pub fn resolve_iterative(fqdn: &str, qtype: u16, nameserver: &str) -> Option<String> {
    let mut rng = rand::thread_rng();
    let id: u16 = rng.gen();

    // 以下の条件に達するまでクエリを投げ続ける
    // - Answer が得られる
    // - RCODE が 0 以外で何らかのエラーが生じている
    let message = message::Message {
        header: message::Header::create(
            id, 0b0, 0b0000, 0b0, 0b0, 0b0, 0b0, 0b000, 0b0000, 0x0001, 0x0000, 0x0000, 0x0000,
        ),
        question: message::Question::new(fqdn, qtype, 0x0001),
    };

    println!("{:?} に問い合わせます...", nameserver);
    sleep(time::Duration::from_millis(2000));

    let socket = UdpSocket::bind("0.0.0.0:12345").expect("Couldn't bind to address");
    let buffer = message.to_bytes();
    let buffer_slice = buffer.as_slice();
    socket
        .send_to(&buffer_slice, [nameserver, ":53"].concat())
        .expect("Couldn't send");

    // Receive
    let mut buf = [0; 512];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
    println!(
        "number_of_bytes: {}, src_addr: {:?}",
        number_of_bytes, src_addr
    );
    println!("{:?}", buf);

    drop(socket);

    // Response
    let mut ret_header: [u8; 12] = Default::default();
    ret_header.copy_from_slice(&buf[0..12]);
    let ret_header = message::Header::parse(&ret_header);

    let ret_body = &buf[12..512];
    let questions = message::Question::parse(&ret_body, ret_header.qd_count.into());
    println!("{:?}", questions.0);

    let ret_body = &buf[(questions.1 + 12)..512];
    let resources = message::Resource::parse(
        &buf,
        ret_body,
        (ret_header.an_count + ret_header.ns_count + ret_header.ar_count).into(),
    );

    println!("{:?}", resources);

    // 判定
    if ret_header.an_count > 0 {
        println!("結果が得られました。終了します");
        let end = ret_header.an_count as usize;
        let an_records = &resources[0..end];

        println!("Answer records: {:?}", an_records);

        match &an_records[0].address {
            message::IpAddr::V4(ipv4) => return Some(ipv4.to_string()),
            message::IpAddr::V6(_) => return None,
        }
    }
    if ret_header.rcode() > 0 {
        println!(
            "エラーが返されました (RCODE: {:?}) 。終了します",
            ret_header.rcode()
        );
        return None;
    }

    println!("ここに答えはありませんでした。次の問い合わせ先を探します");

    // 次の問い合わせ先を探す
    let begin = ret_header.an_count as usize;
    let end = begin + ret_header.ns_count as usize;
    let ns_records = &resources[begin..end];

    println!(
        "{:?} について、 {:?} などが知っているようです。問い合わせてみましょう",
        ns_records[0].name, ns_records[0].nsdname
    );

    if ret_header.ar_count == 0 {
        println!(
            "付加情報部がないので、まず問い合わせ先 {:?} の IP アドレスを調べます。",
            ns_records[0].nsdname
        );
        let address = resolve(ns_records[0].nsdname.as_str(), 1);
        println!("問い合わせ先の IP アドレスは {:?} です。", address);
        match address {
            Some(addr) => return resolve_iterative(fqdn, qtype, addr.as_str()),
            None => todo!(),
        }
    }

    let begin = (ret_header.an_count + ret_header.ns_count) as usize;
    let end = begin + ret_header.ar_count as usize;
    let ar_records = &resources[begin..end];

    for ar_record in ar_records {
        if ar_record.rr_type == 1 {
            match &ar_record.address {
                message::IpAddr::V4(ipv4) => {
                    return resolve_iterative(fqdn, qtype, ipv4);
                }
                message::IpAddr::V6(_) => todo!(),
            }
        }
    }

    return None;
}

pub fn resolve(fqdn: &str, qtype: u16) -> Option<String> {
    println!("{:?} の type {:?} を解決していくよ！", fqdn, qtype);

    let mut rng = rand::thread_rng();
    let nameserver = ROOT_NAME_SERVERS[rng.gen_range(0..ROOT_NAME_SERVERS.len())];

    return resolve_iterative(fqdn, qtype, nameserver);
}
