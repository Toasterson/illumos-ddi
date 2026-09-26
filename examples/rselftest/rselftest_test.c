#include <stdio.h>
#include <stdint.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioccom.h>

struct results { uint32_t passed, failed; uint64_t failed_mask, periodic_ticks; };
#define RSELFTEST_IOC_RESULTS _IOR('S', 1, struct results)
struct ldi_results { uint32_t passed, failed; uint64_t failed_mask; };
#define RSELFTEST_IOC_LDI _IOR('S', 2, struct ldi_results)

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
	close(fd);
	return r.failed != 0 || l.failed != 0;
}
