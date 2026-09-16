# V4L2 capture inventory (research)

Status: **Unproven** in Grannus; no capture card has been exercised. This
inventory records the capability checks required before a backend can claim
support.

Linux V4L2 applications should open a `/dev/videoX` node and call
`VIDIOC_QUERYCAP` first. The kernel documentation defines the
`V4L2_CAP_VIDEO_CAPTURE` (single-planar) and `V4L2_CAP_VIDEO_CAPTURE_MPLANE`
(multi-planar) capability flags, and `V4L2_CAP_STREAMING` for streaming I/O.
The selected path must then enumerate inputs and formats, negotiate dimensions,
pixel format, frame interval, and buffer type, and reject unsupported values
before allocation.

The planned inventory record for each device is: USB/PCI identity and
firmware, kernel and driver versions, node, capture capability flags, supported
pixel formats and intervals, single versus multi-planar support, mmap buffer
count/size, timestamp source, disconnect behavior, and sanitized command/log
evidence. A row enters `docs/hardware-matrix.md` only after a reproducible
device test with timing data.

Source: [Linux kernel VIDIOC_QUERYCAP documentation](https://docs.kernel.org/userspace-api/media/v4l/vidioc-querycap.html)
and [V4L2 capability guidance](https://docs.kernel.org/userspace-api/media/v4l/querycap.html).
