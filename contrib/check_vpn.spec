Name: check_vpn
Version: 0.1.0
Release: 1%{?dist}
Summary: VPN monitoring and automatic reconnect

Group: Applications/System
License: MIT
URL: https://github.com/macg4dave/check_vpn_rust
Source0: %{name}-%{version}.tar.gz

BuildRequires: rust
Requires: /usr/bin

%description
check_vpn monitors the public ISP and runs configured actions when a VPN appears lost.

%prep
# upstream packaging instructions go here

%build
# build with cargo in packager workflow

%install
# install files into %{buildroot}

%files
/usr/bin/check_vpn
/etc/check_vpn/config.xml
/lib/systemd/system/check_vpn.service

%changelog
