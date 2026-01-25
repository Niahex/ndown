use makepad_widgets::*;

live_design! {
    // Nord Color Palette
    // Polar Night
    pub NORD_POLAR_0 = #2E3440
    pub NORD_POLAR_1 = #3B4252
    pub NORD_POLAR_2 = #434C5E
    pub NORD_POLAR_3 = #4C566A

    // Snow Storm
    pub NORD_SNOW_0 = #D8DEE9
    pub NORD_SNOW_1 = #E5E9F0
    pub NORD_SNOW_2 = #ECEFF4

    // Frost
    pub NORD_FROST_0 = #8FBCBB
    pub NORD_FROST_1 = #88C0D0
    pub NORD_FROST_2 = #81A1C1
    pub NORD_FROST_3 = #5E81AC

    // Aurora
    pub NORD_AURORA_RED = #BF616A
    pub NORD_AURORA_ORANGE = #D08770
    pub NORD_AURORA_YELLOW = #EBCB8B
    pub NORD_AURORA_GREEN = #A3BE8C
    pub NORD_AURORA_PURPLE = #B48EAD

    // User Specified Custom Colors
    // Mute: Polar1 at 40% opacity (0.4 * 255 = ~102 = 0x66)
    pub COLOR_MUTE = #3B425266

    // Accent: Frost1 (using standard Nord8 #88C0D0 as primary frost/cyan) at 25% opacity (0.25 * 255 = ~64 = 0x40)
    // Note: If Frost1 refers to the very first item (Nord7 #8FBCBB), use that. Standard Nord "Frost" is usually 88C0D0.
    // I will use #88C0D0 (Nord8) as it is the most common "Nord Accent".
    pub COLOR_ACCENT = #88C0D040

    // Theme Overrides
    pub THEME_COLOR_BG_APP = (NORD_POLAR_0)
    pub THEME_COLOR_BG_CONTAINER = (NORD_POLAR_1)

    pub THEME_COLOR_TEXT_DEFAULT = (NORD_SNOW_2)
    pub THEME_COLOR_TEXT_MUTE = (NORD_SNOW_0)

    pub THEME_COLOR_ACCENT = (NORD_FROST_1)
}
