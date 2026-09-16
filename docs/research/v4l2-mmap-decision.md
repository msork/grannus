# V4L2 mmap decision

Decision: use a narrow Linux FFI boundary rather than adopt `v4l` 0.14.0.
`v4l` is MIT licensed and provides safe mmap streams, but its published release
is several years old. A local boundary keeps ioctl types, buffer counts, and
unsafe invariants auditable and prevents a broad dependency from entering the
platform crate. No hardware support claim is made until dequeue/requeue is
validated on the Elgato device.
