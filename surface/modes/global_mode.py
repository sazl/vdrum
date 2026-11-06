"""Mode 16: Global."""

from surface_spec import *

from components.common import pads


def global_mode():
    m = Layout()
    for c in range(7):
        tr = TRACKS[c]
        m.set(k(1, c + 1), f"{tr} note", GLOBAL, param(f"midi_note{c + 1}"))
        m.set(k(2, c + 1), f"{tr} channel", GLOBAL, param(f"midi_chan_{c + 1}"))
        m.set(f(c + 1), f"{tr} HW fader", MIX, voice_fader(c))
    m.set(k(1, 8), "Tempo", GLOBAL, param("bpm"))
    m.set(k(2, 8), "Global ch.", GLOBAL, param("midi_chan_global"))
    dev = [("LED bright", "led_brightness"), ("Screen bright", "screen_brightness"), ("Popup time", "popup_time"),
           ("Enc. curve", "encoder_curve"), ("Pad velocity", "pad_velocity"), ("List speed", "list_speed"),
           ("Fader mode", "fader_mode"), ("Clock", "clock")]
    for c, (n, s) in enumerate(dev):
        m.set(k(3, c + 1), n, DEVICE, setting(s))
    m.set(f(8), "Global S.rate", GLOBAL, param("voice_decimation_all"))
    tops = [("Sum aux", sum_aux()), ("HW clock", hw_clock()), ("Init kit", init_kit()), ("Save kit", save_kit()),
            ("Compare", compare()), ("Undo", undo()), ("Redo", redo()), ("Panic", panic())]
    for c, (n, d) in enumerate(tops):
        m.set(t(c + 1), n, GLOBAL, d)
    pads(m)
    return m
