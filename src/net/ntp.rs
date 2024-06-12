use std::io::{Cursor, Seek, SeekFrom};
use std::net::UdpSocket;

use byteorder::{BigEndian, ReadBytesExt};
use chrono::prelude::*;

pub struct NtpClient;
impl NtpClient {
    pub fn new() -> NtpClient {
        NtpClient
    }
    pub fn request(self, server: &str) -> anyhow::Result<Response> {
        let client = UdpSocket::bind("0.0.0.0:0")?;
        client.connect(format!("{server}:123"))?;
        let mut request_data = vec![0; 48];
        request_data[0] = 0x1b;
        client.send(&request_data)?;
        let mut buf = [0; 48];
        client.recv(&mut buf).unwrap();
        let ntp_second = self.unpack_ntp_data(&buf)?;
        let unix_second = ntp_second - 2208988800;
        let response = Response {
            unix_time: unix_second,
        };
        Ok(response)
    }
    fn unpack_ntp_data(self, buffer: &[u8; 48]) -> anyhow::Result<u64> {
        let mut reader = Cursor::new(buffer);
        reader.seek(SeekFrom::Current(40)).unwrap();
        let ntp_second = reader.read_u32::<BigEndian>()?;
        Ok(u64::from(ntp_second))
    }
}

pub struct Response {
    pub unix_time: u64,
}

impl Response {
    pub fn format_time(self, fmt: &str) -> anyhow::Result<String> {
        let dt = DateTime::from_timestamp(
            self.unix_time as i64,
            ((self.unix_time % 1000) as u32) * 1_000_000,
        )
        .ok_or(anyhow::Error::msg("translate time failed!"))?;

        let shanghai = FixedOffset::east_opt(8 * 3600).unwrap();
        Ok(format!("{}", dt.with_timezone(&shanghai).format(fmt)))
    }
}
