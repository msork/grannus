# V4L2 Rust API choice

The `v4l` 0.14.0 crate provides safe Video4Linux2 bindings and an
`io::mmap::Stream` abstraction. Its package metadata declares MIT licensing and
the upstream repository is `raymanfx/libv4l-rs`. The latest published release
is several years old, so maintenance risk is **Experimental** until the API is
validated against the current kernel and Elgato device. Its default `v4l2`
feature uses the separately packaged `v4l2-sys-mit` bindings; the optional
`libv4l` feature must not be enabled because it introduces a different native
dependency path.

Source: [`v4l` 0.14.0 metadata](https://docs.rs/crate/v4l/0.14.0) and
[mmap stream documentation](https://docs.rs/v4l/latest/v4l/io/mmap/struct.Stream.html).

Decision: do not add the dependency or claim mmap support until a focused
prototype verifies format negotiation, bounded buffer count, dequeue/requeue,
device timestamps, and clean stream teardown on `/dev/video4`. The existing
platform capability boundary remains the compatibility surface.
