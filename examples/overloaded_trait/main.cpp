#include "./generated.h"

int main() {
  auto s = "string"_rs.to_owned();
  zngur_dbg(s.as_ref());
  zngur_dbg(s.as_ref_u8());

  auto ipv4 = rust::std::net::Ipv4Addr::from_bits(0);
  auto ipv6 = rust::std::net::Ipv6Addr::new_(0, 0, 0, 0, 0, 0, 0, 0);
  rust::std::net::IpAddr ip = rust::std::net::IpAddr::V6(ipv6);

  zngur_dbg(ip.partial_cmp(ipv4));
  zngur_dbg(ip.partial_cmp(ipv6));
}
