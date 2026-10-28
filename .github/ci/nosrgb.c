// SPDX-License-Identifier: GPL-3.0-only
//
// CI-only LD_PRELOAD shim for GUI tests on Xvfb. baseview (the nih-plug editor window) asks
// glXChooseFBConfig for an sRGB-capable framebuffer, and Xvfb's software GLX offers none, so the
// editor cannot open there. This drops GLX_FRAMEBUFFER_SRGB_CAPABLE_ARB (0x20B2) from the request.
// Real X servers with GPU drivers provide such configs; the plugin itself is unchanged.
//
//   cc -shared -fPIC -O2 -o nosrgb.so .github/ci/nosrgb.c -ldl
//   LD_PRELOAD=$PWD/nosrgb.so xvfb-run -a -s "-screen 0 1920x1080x24 +extension GLX" pluginval ...
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stddef.h>

typedef void *(*choose_fn)(void *, int, const int *, int *);

void *glXChooseFBConfig(void *dpy, int screen, const int *attribs, int *n) {
    static choose_fn real = NULL;
    if (!real) real = (choose_fn)dlsym(RTLD_NEXT, "glXChooseFBConfig");
    if (!real) real = (choose_fn)dlsym(dlopen("libGL.so.1", RTLD_NOW | RTLD_GLOBAL), "glXChooseFBConfig");
    if (!real) return NULL;
    int out[256];
    int j = 0;
    for (int i = 0; attribs && attribs[i] != 0 && j < 252; i += 2) {
        if (attribs[i] == 0x20B2) continue;
        out[j++] = attribs[i];
        out[j++] = attribs[i + 1];
    }
    out[j] = 0;
    return real(dpy, screen, out, n);
}
