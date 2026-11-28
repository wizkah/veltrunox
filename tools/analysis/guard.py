"""Runs a command and kills it before it can take the machine down: models can need more
memory than there is, and the system then swaps until it stops responding.

    guard.py COMMAND [ARGUMENT ...]

The command is killed when its memory passes a limit (three quarters of the physical memory,
or VELTRUNOX_ANALYSIS_MEMORY_GB), when the system reports critical memory pressure, or when
the system disk, which holds the swap files, is nearly full.
"""
import ctypes
import os
import shutil
import signal
import subprocess
import sys
import time

GIB = 1024**3
POLL_SECONDS = 0.2
LEAST_FREE_DISK = 1.5 * GIB
# Exit code of a command that was killed here (EX_TEMPFAIL).
STOPPED = 75


def physical_memory():
    return os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES")


class MacOs:
    """Reads what Activity Monitor shows, which counts compressed and swapped memory."""

    def __init__(self):
        self.library = ctypes.CDLL("/usr/lib/libSystem.B.dylib")

    def memory_of(self, process):
        # struct rusage_info_v2: 16 bytes of UUID, then 64-bit counters; the eighth of
        # them is the physical footprint.
        usage = (ctypes.c_uint64 * 64)()
        if self.library.proc_pid_rusage(process, 2, ctypes.byref(usage)) != 0:
            return 0
        return usage[2 + 7]

    def critical(self):
        level, size = ctypes.c_int(0), ctypes.c_size_t(ctypes.sizeof(ctypes.c_int))
        name = b"kern.memorystatus_vm_pressure_level"
        self.library.sysctlbyname(name, ctypes.byref(level), ctypes.byref(size), None, 0)
        return level.value >= 4


class Linux:
    def memory_of(self, process):
        return self.read(f"/proc/{process}/status", ("VmRSS", "VmSwap"))

    def critical(self):
        return self.read("/proc/meminfo", ("MemAvailable",)) < physical_memory() * 0.05

    def read(self, path, names):
        try:
            with open(path) as file:
                lines = [line.split() for line in file]
        except OSError:
            return 0
        return sum(int(line[1]) * 1024 for line in lines if line[0].rstrip(":") in names)


def reason_to_stop(system, process, limit):
    used = system.memory_of(process)
    if used > limit:
        return f"it needs more than {limit / GIB:.1f} GB of memory"
    if system.critical():
        return "the system is running out of memory"
    if shutil.disk_usage(os.path.expanduser("~")).free < LEAST_FREE_DISK:
        return "the system disk is nearly full"
    return None


def run(command, environment=None):
    """Returns the exit code of the command, or STOPPED."""
    system = MacOs() if sys.platform == "darwin" else Linux()
    set_limit = os.environ.get("VELTRUNOX_ANALYSIS_MEMORY_GB")
    limit = float(set_limit) * GIB if set_limit else physical_memory() * 0.75
    child = subprocess.Popen(command, env=environment)

    def stop(*_):
        child.kill()
        sys.exit(STOPPED)

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    while child.poll() is None:
        reason = reason_to_stop(system, child.pid, limit)
        if reason:
            child.kill()
            child.wait()
            print(f"analysis stopped: {reason}", file=sys.stderr, flush=True)
            return STOPPED
        time.sleep(POLL_SECONDS)
    return child.returncode


if __name__ == "__main__":
    sys.exit(run(sys.argv[1:]))
