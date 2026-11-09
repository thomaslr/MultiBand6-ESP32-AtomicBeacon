// =============================================================================
// MultiBand 6 ESP32 Beacon — Full 3D Assembly Preview
// Displays Base, Lid, and Angled Table Stand in Position
// =============================================================================

use <enclosure_base.scad>
use <enclosure_lid.scad>
use <table_stand.scad>

// Enclosure Base (Slate Grey)
color([0.25, 0.25, 0.28])
    enclosure_base();

// Enclosure Lid (Deep Violet / Translucent)
color([0.45, 0.2, 0.65, 0.85])
    translate([0, 0, 12.0])
        enclosure_lid();

// Attachable Table Stand (Carbon Black)
color([0.15, 0.15, 0.18])
    translate([8.5, -12.0, -18.0])
        rotate([-20, 0, 0])
            table_stand();
