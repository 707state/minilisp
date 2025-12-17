#include <errno.h>
#include <fcntl.h>
#include <linux/spi/spidev.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <time.h>
#include <unistd.h>

#include <gpiod.h>

// ====== 硬件/接口配置 ======
#define SPI_DEV         "/dev/spidev0.0"
#define SPI_SPEED_HZ    16000000

#define TFT_W 128
#define TFT_H 128

// 很多 ST7735 模块有可视区偏移：不对就试 (1,2)/(2,1) 等
#define X_OFFSET 0
#define Y_OFFSET 0

// 竖屏/横屏/颜色顺序靠 MADCTL 调
// 常见：0x00/0x60/0xC0/0xA0；有些需要 BGR 位(0x08)
#define MADCTL_VALUE 0x00

#define GPIO_CHIP_PATH  "/dev/gpiochip0"
#define GPIO_DC         25
#define GPIO_RES        24

// ====== ASCII 画面尺寸（适配 128x160）：32x20 字符，每字符 4x8 像素 ======
#define CW 32
#define CH 20
#define GLYPH_W 4
#define GLYPH_H 8

static void msleep(int ms) {
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (long)(ms % 1000) * 1000000L;
    nanosleep(&ts, NULL);
}

// ====== TFT 上下文 ======
struct tft {
    int spi_fd;
    uint32_t spi_speed;
    struct gpiod_line_request *gpio_req; // DC+RES
};

static int spi_write_bytes(int fd, const uint8_t *buf, size_t len, uint32_t speed) {
    struct spi_ioc_transfer tr;
    memset(&tr, 0, sizeof(tr));
    tr.tx_buf = (unsigned long)buf;
    tr.len = (uint32_t)len;
    tr.speed_hz = speed;
    tr.bits_per_word = 8;
    return ioctl(fd, SPI_IOC_MESSAGE(1), &tr);
}

static struct gpiod_line_request *
request_output_lines(const char *chip_path,
                     const unsigned int *offsets, size_t n,
                     enum gpiod_line_value initial_value,
                     const char *consumer)
{
    struct gpiod_request_config *req_cfg = NULL;
    struct gpiod_line_request *request = NULL;
    struct gpiod_line_settings *settings = NULL;
    struct gpiod_line_config *line_cfg = NULL;
    struct gpiod_chip *chip = NULL;
    int ret;

    chip = gpiod_chip_open(chip_path);
    if (!chip) return NULL;

    settings = gpiod_line_settings_new();
    if (!settings) goto out;

    gpiod_line_settings_set_direction(settings, GPIOD_LINE_DIRECTION_OUTPUT);
    gpiod_line_settings_set_output_value(settings, initial_value);

    line_cfg = gpiod_line_config_new();
    if (!line_cfg) goto out;

    ret = gpiod_line_config_add_line_settings(line_cfg, offsets, n, settings);
    if (ret) goto out;

    req_cfg = gpiod_request_config_new();
    if (!req_cfg) goto out;
    if (consumer) gpiod_request_config_set_consumer(req_cfg, consumer);

    request = gpiod_chip_request_lines(chip, req_cfg, line_cfg);

out:
    if (req_cfg) gpiod_request_config_free(req_cfg);
    if (line_cfg) gpiod_line_config_free(line_cfg);
    if (settings) gpiod_line_settings_free(settings);
    if (chip) gpiod_chip_close(chip);
    return request;
}

static inline void gpio_set(struct tft *t, unsigned int line, int v) {
    enum gpiod_line_value value = v ? GPIOD_LINE_VALUE_ACTIVE : GPIOD_LINE_VALUE_INACTIVE;
    if (gpiod_line_request_set_value(t->gpio_req, line, value) < 0) {
        fprintf(stderr, "gpiod set line %u failed: %s\n", line, strerror(errno));
    }
}

static inline void dc_cmd(struct tft *t)  { gpio_set(t, GPIO_DC, 0); }
static inline void dc_data(struct tft *t) { gpio_set(t, GPIO_DC, 1); }

static void tft_write_cmd(struct tft *t, uint8_t cmd) {
    dc_cmd(t);
    if (spi_write_bytes(t->spi_fd, &cmd, 1, t->spi_speed) < 0)
        perror("spi write cmd");
}

static void tft_write_data(struct tft *t, const uint8_t *data, size_t len) {
    dc_data(t);
    if (spi_write_bytes(t->spi_fd, data, len, t->spi_speed) < 0)
        perror("spi write data");
}

static void tft_write_u16(struct tft *t, uint16_t v) {
    uint8_t b[2] = { (uint8_t)(v >> 8), (uint8_t)(v & 0xFF) };
    tft_write_data(t, b, 2);
}

static void tft_reset(struct tft *t) {
    gpio_set(t, GPIO_RES, 1);
    msleep(10);
    gpio_set(t, GPIO_RES, 0);
    msleep(50);
    gpio_set(t, GPIO_RES, 1);
    msleep(120);
}

static void tft_set_addr_window(struct tft *t, int x0, int y0, int x1, int y1) {
    x0 += X_OFFSET; x1 += X_OFFSET;
    y0 += Y_OFFSET; y1 += Y_OFFSET;

    tft_write_cmd(t, 0x2A);
    tft_write_u16(t, (uint16_t)x0);
    tft_write_u16(t, (uint16_t)x1);

    tft_write_cmd(t, 0x2B);
    tft_write_u16(t, (uint16_t)y0);
    tft_write_u16(t, (uint16_t)y1);

    tft_write_cmd(t, 0x2C);
}

static void tft_init_min(struct tft *t) {
    tft_reset(t);

    tft_write_cmd(t, 0x01); // SWRESET
    msleep(150);

    tft_write_cmd(t, 0x11); // SLPOUT
    msleep(150);

    tft_write_cmd(t, 0x3A); // COLMOD
    uint8_t fmt = 0x05;     // RGB565
    tft_write_data(t, &fmt, 1);
    msleep(10);

    tft_write_cmd(t, 0x36); // MADCTL
    uint8_t madctl = MADCTL_VALUE;
    tft_write_data(t, &madctl, 1);
    msleep(10);

    tft_write_cmd(t, 0x29); // DISPON
    msleep(120);
}

// ====== 画点/刷屏（RGB565） ======
static void tft_push_frame_rgb565(struct tft *t, const uint16_t *frame) {
    tft_set_addr_window(t, 0, 0, TFT_W - 1, TFT_H - 1);

    // 分块发（每次发 1024 字节左右）
    uint8_t buf[1024];
    size_t total_px = (size_t)TFT_W * (size_t)TFT_H;
    size_t idx = 0;

    while (idx < total_px) {
        size_t px_this = (sizeof(buf) / 2);
        if (idx + px_this > total_px) px_this = total_px - idx;

        for (size_t i = 0; i < px_this; i++) {
            uint16_t v = frame[idx + i];
            buf[2*i]   = (uint8_t)(v >> 8);
            buf[2*i+1] = (uint8_t)(v & 0xFF);
        }
        tft_write_data(t, buf, px_this * 2);
        idx += px_this;
    }
}

// ====== 极小 4x8 字模，只包含 donut 用到的字符集 " .,-~:;=!*#$@" + 空格 ======
static const char charset[] = " .,-~:;=!*#$@";
// 每个 glyph 8 行，每行 4 bit（低 4 位有效，从高到低画）
static const uint8_t glyphs[][8] = {
    // ' ' (space)
    {0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0},
    // '.'
    {0x0,0x0,0x0,0x0,0x0,0x0,0x6,0x6},
    // ','
    {0x0,0x0,0x0,0x0,0x0,0x6,0x6,0x4},
    // '-'
    {0x0,0x0,0x0,0x0,0xF,0x0,0x0,0x0},
    // '~'
    {0x0,0x0,0x0,0xA,0x5,0x0,0x0,0x0},
    // ':'
    {0x0,0x6,0x6,0x0,0x0,0x6,0x6,0x0},
    // ';'
    {0x0,0x6,0x6,0x0,0x0,0x6,0x6,0x4},
    // '='
    {0x0,0x0,0xF,0x0,0xF,0x0,0x0,0x0},
    // '!'
    {0x4,0x4,0x4,0x4,0x4,0x0,0x4,0x0},
    // '*'
    {0x0,0xA,0xF,0xA,0x0,0x0,0x0,0x0},
    // '#'
    {0x0,0xA,0xF,0xA,0xF,0xA,0x0,0x0},
    // '$'
    {0x4,0x7,0xC,0x7,0x1,0xE,0x4,0x0},
    // '@'
    {0x6,0x9,0xB,0xB,0x8,0x7,0x0,0x0},
};

static int glyph_index(char c) {
    const char *p = strchr(charset, c);
    if (!p) return 0;
    return (int)(p - charset);
}

static void draw_char(uint16_t *fb, int cx, int cy, char ch, uint16_t fg, uint16_t bg) {
    int gi = glyph_index(ch);
    int px0 = cx * GLYPH_W;
    int py0 = cy * GLYPH_H;
    for (int row = 0; row < GLYPH_H; row++) {
        uint8_t bits = glyphs[gi][row] & 0x0F;
        for (int col = 0; col < GLYPH_W; col++) {
            int on = (bits >> (GLYPH_W - 1 - col)) & 1;
            int x = px0 + col;
            int y = py0 + row;
            if ((unsigned)x < TFT_W && (unsigned)y < TFT_H) {
                fb[y * TFT_W + x] = on ? fg : bg;
            }
        }
    }
}

static void draw_ascii_buffer(uint16_t *fb, const char *buf, uint16_t fg, uint16_t bg) {
    // 背景先清一遍
    for (int i = 0; i < TFT_W * TFT_H; i++) fb[i] = bg;

    for (int y = 0; y < CH; y++) {
        for (int x = 0; x < CW; x++) {
            draw_char(fb, x, y, buf[x + CW * y], fg, bg);
        }
    }
}

// ====== 打开/关闭 ======
static int tft_open(struct tft *t) {
    memset(t, 0, sizeof(*t));
    t->spi_speed = SPI_SPEED_HZ;

    t->spi_fd = open(SPI_DEV, O_RDWR);
    if (t->spi_fd < 0) { perror("open spidev"); return -1; }

    uint8_t mode = SPI_MODE_0;
    uint8_t bits = 8;
    if (ioctl(t->spi_fd, SPI_IOC_WR_MODE, &mode) < 0) { perror("SPI_IOC_WR_MODE"); return -1; }
    if (ioctl(t->spi_fd, SPI_IOC_WR_BITS_PER_WORD, &bits) < 0) { perror("SPI_IOC_WR_BITS_PER_WORD"); return -1; }
    if (ioctl(t->spi_fd, SPI_IOC_WR_MAX_SPEED_HZ, &t->spi_speed) < 0) { perror("SPI_IOC_WR_MAX_SPEED_HZ"); return -1; }

    unsigned int lines[2] = { GPIO_DC, GPIO_RES };
    t->gpio_req = request_output_lines(GPIO_CHIP_PATH, lines, 2,
                                       GPIOD_LINE_VALUE_INACTIVE,
                                       "st7735_donut");
    if (!t->gpio_req) {
        fprintf(stderr, "failed to request gpio lines: %s\n", strerror(errno));
        return -1;
    }

    gpio_set(t, GPIO_RES, 1);
    gpio_set(t, GPIO_DC, 0);
    return 0;
}

static void tft_close(struct tft *t) {
    if (t->gpio_req) gpiod_line_request_release(t->gpio_req);
    if (t->spi_fd >= 0) close(t->spi_fd);
}

// ====== 主程序：donut -> ASCII buf -> RGB565 fb -> 推屏 ======
int main(void) {
    struct tft t;
    if (tft_open(&t) != 0) return 1;

    tft_init_min(&t);

    float A = 0, B = 0;
    float z[CW * CH];
    char  b[CW * CH];

    static uint16_t fb[TFT_W * TFT_H];

    for (;;) {
        memset(b, ' ', sizeof(b));
        memset(z, 0, sizeof(z));

        // 下面基本是你 donut.c 的算法，只是把 80x22 改成 CWxCH
        for (float j = 0; j < 6.28f; j += 0.07f) {
            for (float i = 0; i < 6.28f; i += 0.02f) {
                float c = sinf(i);
                float d = cosf(j);
                float e = sinf(A);
                float f = sinf(j);
                float g = cosf(A);
                float h = d + 2.0f;
                float D = 1.0f / (c * h * e + f * g + 5.0f);
                float l = cosf(i);
                float m = cosf(B);
                float n = sinf(B);
                float t2 = c * h * g - f * e;

                // 根据 CW/CH 调整投影比例（原来 40/12/30/15）
                int x = (CW / 2) + (int)((CW * 0.38f) * D * (l * h * m - t2 * n));
                int y = (CH / 2) + (int)((CH * 0.62f) * D * (l * h * n + t2 * m));
                int o = x + CW * y;

                int N = (int)(8.0f * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n));

                if (y >= 0 && y < CH && x >= 0 && x < CW && D > z[o]) {
                    z[o] = D;
                    const char *shades = ".,-~:;=!*#$@";
                    b[o] = shades[(N > 0 ? N : 0) % 12];
                }
            }
        }

        // ASCII -> 像素
        draw_ascii_buffer(fb, b, 0xFFFF /*白*/, 0x0000 /*黑*/);

        // 推送到屏幕
        tft_push_frame_rgb565(&t, fb);

        // 动起来
        A += 0.04f;
        B += 0.02f;

        usleep(30000);
    }

    tft_close(&t);
    return 0;
}
