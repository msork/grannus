# systemd packaging

The unit is a future hardened template and does not work yet because `serve`
is intentionally unimplemented. Do not install/enable it until configuration,
pairing, device permissions, and required capabilities are defined and tested.
Prefer udev ACLs and narrowly scoped helper/service privileges over running the
whole host as root.
