#include <stdio.h>
#include <stdint.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <sys/ioccom.h>
#include <sys/stat.h>

struct results { uint32_t passed, failed; uint64_t failed_mask, periodic_ticks; };
#define RSELFTEST_IOC_RESULTS _IOR('S', 1, struct results)
struct ldi_results { uint32_t passed, failed; uint64_t failed_mask; };
#define RSELFTEST_IOC_LDI _IOR('S', 2, struct ldi_results)
struct anonfd_results { uint32_t passed, failed; uint64_t failed_mask; int32_t other_fd, fd; };
#define RSELFTEST_IOC_ANONFD _IOWR('S', 3, struct anonfd_results)

int main(int argc, char **argv) {
	const char *path = argc > 1 ? argv[1] : "/devices/pseudo/rselftest@0:rselftest";
	int fd = open(path, O_RDONLY);
	if (fd < 0) { perror(path); return 1; }
	struct results r, r2;
	if (ioctl(fd, RSELFTEST_IOC_RESULTS, &r) != 0) { perror("RESULTS"); return 1; }
	printf("passed=%u failed=%u failed_mask=0x%llx periodic_ticks=%llu\n", r.passed, r.failed,
	    (unsigned long long)r.failed_mask, (unsigned long long)r.periodic_ticks);
	sleep(1);
	if (ioctl(fd, RSELFTEST_IOC_RESULTS, &r2) != 0) { perror("RESULTS"); return 1; }
	printf("periodic ticks in one second: %llu\n", (unsigned long long)(r2.periodic_ticks - r.periodic_ticks));
	if (ioctl(fd, _IO('S', 99)) == 0) { printf("unknown ioctl unexpectedly succeeded\n"); return 1; }
	perror("unknown ioctl (expect ENOTTY)");
	struct ldi_results l;
	if (ioctl(fd, RSELFTEST_IOC_LDI, &l) != 0) { perror("LDI"); return 1; }
	printf("ldi passed=%u failed=%u failed_mask=0x%llx\n", l.passed, l.failed, (unsigned long long)l.failed_mask);
	struct anonfd_results a = { 0 };
	int bad = 0;
	a.other_fd = open("/dev/null", O_RDONLY);
	a.fd = -1;
	if (ioctl(fd, RSELFTEST_IOC_ANONFD, &a) != 0) {
		if (errno != ENOTSUP) { perror("ANONFD"); return 1; }
		printf("anonfd: not built (private-gate feature off)\n");
	} else {
		struct stat s1, s2;
		int ok = a.fd >= 0 && fstat(a.fd, &s1) == 0 && fstat(fd, &s2) == 0 &&
		    s1.st_rdev == s2.st_rdev && (fcntl(a.fd, F_GETFD) & FD_CLOEXEC) != 0;
		printf("anonfd passed=%u failed=%u failed_mask=0x%llx new fd %d: %s\n", a.passed, a.failed,
		    (unsigned long long)a.failed_mask, a.fd, ok ? "same device, close-on-exec" : "WRONG");
		bad = a.failed != 0 || !ok;
		if (a.fd >= 0)
			close(a.fd);
	}
	close(a.other_fd);
	close(fd);
	return r.failed != 0 || l.failed != 0 || bad;
}
